/// This is used by [`Loader::MakeMut`] and [`Loader::CleanCopiedFlags`]
const COPIED: Flags = Flags::BIT_9;

pub fn LoadKernel(
   kernel: Kernel<'_>,
   pageTable: &mut (impl MapperAllSizes + Translate),
   frameAllocator: &mut impl FrameAllocator<Size4KiB>,
   usedEntries: &mut UsedLevel4Entries,
) -> Result<(VirtAddr, VirtAddr, Option<TlsTemplate>), &'static str> {
   let mut loader = Loader::new(kernel, pageTable, frameAllocator, usedEntries)?;
   let tlsTemplate = loader.LoadSegments()?;

   return Ok((
      VirtAddr::new(loader.virtualAddressOffset.Offset() as u64),
      loader.EntryPoint(),
      tlsTemplate,
   ));
}

/// Verify that the virtual address offset belongs to a load segment.
pub fn CheckLoadOffset(elfFile: &ElfFile, offset: u64) -> Result<(), &'static str> {
   for procHeader in elfFile.program_iter() {
      if let Type::Load = procHeader.get_type()? {
         if procHeader.virtual_addr() <= offset {
            let segmentOffset = offset - procHeader.virtual_addr();
            if segmentOffset < procHeader.mem_size() {
               return Ok(());
            }
         }
      }
   }

   return Err("offset is not in load segment");
}

pub struct Loader<'a, M, F> {
   elfFile: ElfFile<'a>,
   frameAllocator: &'a mut F,
   kernelOffset: PhysAddr,
   pageTable: &'a mut M,
   virtualAddressOffset: VirtualAddressOffset,
}

impl<'a, M, F> Loader<'a, M, F>
where
   M: MapperAllSizes + Translate,
   F: FrameAllocator<Size4KiB>,
{
   pub fn new(
      kernel: Kernel<'a>,
      pageTable: &'a mut M,
      frameAllocator: &'a mut F,
      usedEntries: &mut UsedLevel4Entries,
   ) -> Result<Self, &'static str> {
      log::info!("ELF file loaded at {:#p}", kernel.elf.input);
      let kernelOffset = PhysAddr::new(&kernel.elf.input[0] as *const u8 as u64);
      if !kernelOffset.is_aligned(PAGE_SIZE) {
         return Err("Loaded kernel ELF file is not sufficiently aligned");
      }

      let elfFile = kernel.elf;
      for procHeader in elfFile.program_iter() {
         program::sanity_check(procHeader, &elfFile)?;
      }

      let virtualAddressOffset = match elfFile.header.pt2.type_().as_type() {
         header::Type::None => unimplemented!(),
         header::Type::Relocatable => unimplemented!(),
         header::Type::Executable => VirtualAddressOffset::zero(),
         header::Type::SharedObject => {
            let loadProcHeaders = elfFile
               .program_iter()
               .filter(|h| matches!(h.get_type(), Ok(Type::Load)));

            let maxAddress = loadProcHeaders
               .clone()
               .map(|h| h.virtual_addr() + h.mem_size())
               .max()
               .unwrap_or(0);

            let minAddress = loadProcHeaders
               .clone()
               .map(|h| h.virtual_addr())
               .min()
               .unwrap_or(0);

            let size = maxAddress - minAddress;
            let align = loadProcHeaders.map(|h| h.align()).max().unwrap_or(1);

            let offset = usedEntries.GetFreeAddress(size, align).as_u64();
            VirtualAddressOffset::new(i128::from(offset) - i128::from(minAddress))
         }
         header::Type::Core => unimplemented!(),
         header::Type::ProcessorSpecific(_) => unimplemented!(),
      };

      log::info!(
         "Virtual address offset: {:#x}",
         virtualAddressOffset.Offset()
      );
      usedEntries.MarkSegments(elfFile.program_iter(), virtualAddressOffset);

      header::sanity_check(&elfFile)?;
      return Ok(Loader {
         elfFile,
         frameAllocator,
         kernelOffset,
         pageTable,
         virtualAddressOffset,
      });
   }

   pub fn LoadSegments(&mut self) -> Result<Option<TlsTemplate>, &'static str> {
      let mut tlsTemplate: Option<TlsTemplate> = None;
      for procHeader in self.elfFile.program_iter() {
         match procHeader.get_type()? {
            Type::Load => self.HandleLoadSegment(procHeader),
            Type::Tls => {
               if tlsTemplate.is_none() {
                  tlsTemplate = Some(self.HandleTlsSegment(procHeader)?);
               } else {
                  return Err("multiple TLS segments not supported");
               }
            }
            Type::Null
            | Type::Dynamic
            | Type::Interp
            | Type::Note
            | Type::ShLib
            | Type::Phdr
            | Type::GnuRelro
            | Type::OsSpecific(_)
            | Type::ProcessorSpecific(_) => {}
         }
      }

      // Apply relocations in virtual memory
      for procHeader in self.elfFile.program_iter() {
         if let Type::Dynamic = procHeader.get_type()? {
            self.HandleDynamicSegment(procHeader, &self.elfFile)?
         }
      }

      self.CleanCopiedFlags(&self.elfFile).unwrap();
      return Ok(tlsTemplate);
   }

   pub fn EntryPoint(&self) -> VirtAddr {
      return VirtAddr::new(self.virtualAddressOffset + self.elfFile.header.pt2.entry_point());
   }

   pub fn HandleLoadSegment(&mut self, segment: ProgramHeader) -> Result<(), &'static str> {
      log::info!("Handling segment: {:?}", segment);

      let physStartAddress = self.kernelOffset + segment.offset();
      let startFrame = PhysFrame::containing_address(physStartAddress);
      let endFrame = PhysFrame::containing_address(physStartAddress + segment.file_size() - 1u64);

      let virtStartAddress = VirtAddr::new(self.virtualAddressOffset + segment.virtual_addr());
      let startPage = Page::containing_address(virtStartAddress);

      let mut segmentFlags = Flags::PRESENT;
      if !segment.flags().is_execute() {
         segmentFlags |= Flags::NO_EXECUTE;
      }

      if segment.flags().is_write() {
         segmentFlags |= Flags::WRITABLE;
      }

      for frame in PhysFrame::range_inclusive(startFrame, endFrame) {
         let offset = frame - startFrame;
         let page = startPage + offset;
         let flusher = unsafe {
            self
               .pageTable
               .map_to(page, frame, segmentFlags, self.frameAllocator)
               .map_err(|| "map to failed")?
         };

         // We are operating on an inactive page table, so no need to flush anything.
         flusher.ignore();
      }

      if segment.mem_size() > segment.file_size() {
         // .bss section (or similar), which needs to be mapped and zeroed
         self.HandleBssSection(&segment, segmentFlags)?;
      }

      return Ok(());
   }

   pub fn HandleBssSegment(
      &mut self,
      segment: &ProgramHeader,
      flags: Flags,
   ) -> Result<(), &'static str> {
      log::info!("Handling BSS segment: {:?}", segment);

      let virtStartAddress = VirtAddr::new(self.virtualAddressOffset + segment.virtual_addr());
      let memSize = segment.mem_size();
      let fileSize = segment.file_size();

      // calculate virtual memory region that must be zeroed
      let zeroStart = virtStartAddress + fileSize;
      let zeroEnd = virtStartAddress + memSize;

      // a type alias that helps in efficiently clearing a page
      type PageArray = [u64; Size4KiB::SIZE as usize / 8];
      const ZERO_ARRAY: PageArray = [0; Size4KiB::SIZE as usize / 8];

      // In some cases, `zero_start` might not be page-aligned. This requires some
      // special treatment because we can't safely zero a frame of the original file.
      let dataBytesBeforeZero = zeroStart.as_u64() & 0xfff;
      if dataBytesBeforeZero != 0 {
         // The last non-bss frame of the segment consists partly of data and partly of bss
         // memory, which must be zeroed. Unfortunately, the file representation might have
         // reused the part of the frame that should be zeroed to store the next segment. This
         // means that we can't simply overwrite that part with zeroes, as we might overwrite
         // other data this way.
         //
         // Example:
         //
         //   XXXXXXXXXXXXXXX000000YYYYYYY000ZZZZZZZZZZZ     virtual memory (XYZ are data)
         //   |·············|     /·····/   /·········/
         //   |·············| ___/·····/   /·········/
         //   |·············|/·····/‾‾‾   /·········/
         //   |·············||·····|/·̅·̅·̅·̅·̅·····/‾‾‾‾
         //   XXXXXXXXXXXXXXXYYYYYYYZZZZZZZZZZZ              file memory (zeros are not saved)
         //   '       '       '       '        '
         //   The areas filled with dots (`·`) indicate a mapping between virtual and file
         //   memory. We see that the data regions `X`, `Y`, `Z` have a valid mapping, while
         //   the regions that are initialized with 0 have not.
         //
         //   The ticks (`'`) below the file memory line indicate the start of a new frame. We
         //   see that the last frames of the `X` and `Y` regions in the file are followed
         //   by the bytes of the next region. So we can't zero these parts of the frame
         //   because they are needed by other memory regions.
         //
         // To solve this problem, we need to allocate a new frame for the last segment page
         // and copy all data content of the original frame over. Afterwards, we can zero
         // the remaining part of the frame since the frame is no longer shared with other
         // segments now.

         let lastPage = Page::containing_address(virtStartAddress + fileSize - 1u64);
         let newFrame = unsafe { self.MakeMut(lastPage) };
         let newBytesPointer = newFrame.start_address().as_u64() as *mut u8;
         unsafe {
            core::ptr::write_bytes(
               newBytesPointer.add(dataBytesBeforeZero as usize),
               0,
               (Size4KiB::SIZE - dataBytesBeforeZero) as usize,
            )
         }
      }

      // Map additional frames for `.bss` memory that is not present in source file.
      let startPage =
         Page::containing_address(VirtAddr::new(align_up(zeroStart.as_u64(), Size4KiB::SIZE)));

      let endPage = Page::containing_address(zeroEnd - 1u64);
      for page in Page::range_inclusive(startPage, endPage) {
         // Allocate a new, unused frame.
         let frame = self.frameAllocator.allocate_frame().unwrap();

         // Zero out frame, utilising identity mapping.
         let framePointer = frame.start_address().as_u64() as *mut PageArray;
         unsafe { framePointer.write(ZERO_ARRAY) };

         // Map the frame.
         let flusher = unsafe {
            self
               .pageTable
               .map_to(page, frame, flags, self.frameAllocator)
               .map_err(|_err| "Failed to keep the new frame for .bss memory")?
         };

         // We are operating on an inactive page table, so we do not need to flush our changes.
         flusher.ignore();
      }

      return Ok(());
   }

   pub fn HandleTlsSegment(&mut self, segment: ProgramHeader) -> Result<TlsTemplate, &'static str> {
      return Ok(TlsTemplate {
         start: self.virtualAddressOffset + segment.virtual_addr(),
         memSize: segment.mem_size(),
         fileSize: segment.file_size(),
      });
   }

   pub fn HandleDynamicSegment(
      &mut self,
      segment: ProgramHeader,
      elfFile: &ElfFile,
   ) -> Result<(), &'static str> {
      let data = segment.get_data(elfFile)?;
      let data = if let SegmentData::Dynamic64(data) = data {
         data
      } else {
         panic!("expected Dynamic64 segment")
      };

      let mut rela = None;
      let mut relaSize = None;
      let mut relaEnt = None;
      for rel in data {
         let tag = rel.get_tag()?;
         match tag {
            dynamic::Tag::Rela => {
               let pointer = rel.get_ptr()?;
               let previous = rela.replace(pointer);
               if previous.is_some() {
                  return Err("Dynamic section contains more than one Rela entry");
               }
            }
            dynamic::Tag::RelaSize => {
               let value = rel.get_val()?;
               let previous = relaSize.replace(value);
               if previous.is_some() {
                  return Err("Dynamic section contains more than one RelaSize entry");
               }
            }
            dynamic::Tag::RelaEnt => {
               let value = rel.get_val()?;
               let previous = relaEnt.replace(value);
               if previous.is_some() {
                  return Err("Dynamic section contains more than one RelaEnt entry");
               }
            }
            _ => {}
         }
      }

      let offset = if let Some(rela) = rela {
         rela
      } else {
         // The section does not contain any relocations
         if relaSize.is_some() || relaEnt.is_some() {
            return Err("Rela entry is missing but RelaSize and RelaEnt have been provided");
         }

         return Ok(());
      };

      let totalSize = relaSize.ok_or("RelaSize entry is missing")?;
      let entrySize = relaEnt.ok_or("RelaEnt entry is missing")?;

      // Ensure that the reported size matches our `Rela<u64>`.
      assert_eq!(
         entrySize,
         size_of::<Rela<u64>>() as u64,
         "unsupported entry size: {entrySize}"
      );

      // Apply the relocations.
      let numEntries = totalSize / entrySize;
      for index in 0..numEntries {
         let rela = self.ReadRelocation(offset, index);
         self.ApplyRelocation(rela, elfFile)?;
      }

      return Ok(());
   }

   /// This method is intended for making the memory loaded by a Load segment mutable.
   ///
   /// All memory from a Load segment starts out by mapped to the same frames that
   /// contain the ELF file. Thus writing to memory in that state will cause aliasing issues.
   /// To avoid that, we allocate a new frame, copy all bytes from the old frame to the new frame,
   /// and remap the page to the new frame. At this point the page no longer aliases the elf file
   /// and we can write to it.
   ///
   /// When we map the new frame we also set [`COPIED`] flag in the page table flags, so that
   /// we can detect if the frame has already been copied when we try to modify the page again.
   ///
   /// ## Safety
   /// - `page` should be a page mapped by a Load segment.
   ///
   /// ## Panics
   /// Panics if the page is not mapped in `self.pageTable`.
   pub unsafe fn MakeMut(&mut self, page: Page) -> PhysFrame {
      let (frame, flags) = match self.pageTable.translate(page.start_address()) {
         TranslateResult::Mapped { frame, flags, .. } => (frame, flags),
         TranslateResult::NotMapped => panic!("{:?} is not mapped", page),
         TranslateResult::InvalidFrameAddress(_) => unreachable!(),
      };

      let frame = if let MappedFrame::Size4KiB(frame) = frame {
         frame
      } else {
         // We only map 4k pages
         unreachable!()
      };

      if flags.contains(COPIED) {
         // The frame was already copied, and we are free to modify it.
         return frame;
      }

      // Allocate a new frame and copy the memory, utilising both frames being identity mapped.
      let newFrame = self.frameAllocator.allocate_frame().unwrap();
      let framePointer = frame.start_address().as_u64() as *const u8;
      let newFramePointer = newFrame.start_address().as_u64() as *mut u8;
      unsafe {
         core::ptr::copy_nonoverlapping(framePointer, newFramePointer, Size4KiB::SIZE as usize);
      }

      // Replace the underlying frame and update the flags.
      self.pageTable.unmap(page).unwrap().1.ignore();
      let newFlags = flags | COPIED;
      unsafe {
         self
            .pageTable
            .map_to(page, newFrame, newFlags, self.frameAllocator)
            .unwrap()
            .ignore();
      }

      return newFrame;
   }

   /// Removes the custom flags set by [`Loader::MakeMut`].
   pub fn CleanCopiedFlags(&mut self, elfFile: &ElfFile) -> Result<(), &'static str> {
      for procHeader in elfFile.program_iter() {
         if let Type::Load = procHeader.get_type()? {
            let start = VirtAddr::new(self.virtualAddressOffset + procHeader.virtual_addr());
            let end = VirtAddr::new(
               self.virtualAddressOffset + procHeader.virtual_addr() + procHeader.mem_size(),
            );

            let startPage = Page::containing_address(start);
            let endPage = Page::containing_address(end - 1u64);

            for page in Page::<Size4KiB>::range_inclusive(startPage, endPage) {
               let res = self.pageTable.translate(page.start_address());
               let flags = match res {
                  TranslateResult::Mapped {
                     frame: _,
                     offset: _,
                     flags,
                  } => flags,
                  TranslateResult::NotMapped | TranslateResult::InvalidFrameAddress(_) => {
                     unreachable!("has the elf file not been correctly mapped?")
                  }
               };

               if flags.contains(COPIED) {
                  // Remove the flag.
                  unsafe {
                     self
                        .pageTable
                        .update_flags(page, flags & !COPIED)
                        .unwrap()
                        .ignore();
                  }
               }
            }
         }
      }

      return Ok(());
   }

   /// Reads a relocation from the relocation table.
   pub fn ReadRelocation(&self, relocationTable: u64, idx: u64) -> Rela<u64> {
      // Calculate the source of the entry in the relocation table.
      let offset = relocationTable + size_of::<Rela<u64>>() as u64 * idx;
      let value = self.virtualAddressOffset + offset;
      let address =
         VirtAddr::try_new(value).expect("relocation table is outside the address space");

      let mut buffer = [0; 24];
      self.copyFrom(address, &mut buffer);

      // Convert the bytes read into `Rela<u64>`
      return unsafe {
         // SAFETY: Any bit pattern is valid for `Rela<u64>` and buf is
         // valid for reads.
         core::ptr::read_unaligned(&buffer as *const u8 as *const Rela<u64>)
      };
   }

   pub fn ApplyRelocation(
      &mut self,
      rela: Rela<u64>,
      elfFile: &ElfFile,
   ) -> Result<(), &'static str> {
      let symbolIndex = rela.get_symbol_table_index();
      assert_eq!(
         symbolIndex, 0,
         "relocations using the symbol table are not supported"
      );

      match rela.get_type() {
         // R_AMD64_RELATIVE
         8 => {
            // Ensure the relocation happens in memory mapped by a Load segment.
            CheckLoadOffset(elfFile, rela.get_offset())?;

            // Calculate the destination of the relocation.
            let address = self.virtualAddressOffset + rela.get_offset();
            let address = VirtAddr::new(address);

            // Calculate the relocated value
            let value = self.virtualAddressOffset + rela.get_addend();

            unsafe {
               // SAFETY: we have verified the address is within a Load segment.
               self.copyTo(address, &value.to_ne_bytes());
            }
         }

         ty => unimplemented!(),
      }

      return Ok(());
   }

   /// Mark a region of memory indicated by a GNU_RELRO segment as read-only.
   ///
   /// This is a security mitigation used to protect memory regions that
   /// need to be writable while applying relocations, but should never be
   /// written to after relocations have been applied.
   pub fn HandleRelroSegment(&mut self, segment: ProgramHeader) {
      let start = self.virtualAddressOffset + segment.virtual_addr();
      let end = start + segment.mem_size();

      let start = VirtAddr::new(start);
      let end = VirtAddr::new(end);

      let startPage = Page::containing_address(start);
      let endPage = Page::containing_address(end - 1u64);
      for page in Page::<Size4KiB>::range_inclusive(startPage, endPage) {
         // Translate the page and get the flags.
         let res = self.pageTable.translate(page.start_address());
         let flags = match res {
            TranslateResult::Mapped {
               frame: _,
               offset: _,
               flags,
            } => flags,
            TranslateResult::NotMapped | TranslateResult::InvalidFrameAddress(_) => {
               unreachable!("has the elf file not been correctly mapped?")
            }
         };

         if flags.contains(Flags::WRITABLE) {
            // Remove the WRITABLE flag.
            unsafe {
               self
                  .pageTable
                  .update_flags(page, flags & !Flags::WRITABLE)
                  .unwrap()
                  .ignore();
            }
         }
      }
   }

   /// Copy from the kernel address space.
   ///
   /// ## Panics
   ///
   /// Panics if a page has not been mapped yet in `self.pageTable`.
   fn copyFrom(&self, address: VirtAddr, buffer: &mut [u8]) {
      // We can't know for sure that contiguous virtual address are contiguous
      // in physical memory, so we iterate of the pages spanning the
      // addresses, translate them to frames and copy the data.

      let endInclusiveAddress = Step::forward_checked(address, buffer.len() - 1)
         .expect("end address outside of the virtual address space");

      let startPage = Page::<Size4KiB>::containing_address(address);
      let endInclusivePage = Page::<Size4KiB>::containing_address(endInclusiveAddress);

      for page in startPage..=endInclusivePage {
         // Translate the virtual page to the physical frame.
         let physAddress = self
            .pageTable
            .translate_page(page)
            .expect("address is not mapped to the kernel's memory space");

         // Find which address range we want to copy from the frame.

         // This page covers these addresses:
         let pageBegin = page.start_address();
         let pageEndInclusive = page.start_address() + 4095u64;

         // We wish to copy from the following address in the frame:
         let startCopyAddress = cmp::max(address, pageBegin);
         let endCopyAddressInclusive = cmp::min(endInclusiveAddress, pageEndInclusive);

         // These are the offsets into the frame we want to copy from:
         let startOffsetInFrame = (startCopyAddress - pageBegin) as usize;
         let endOffsetInFrameInclusive = (endCopyAddressInclusive - pageBegin) as usize;

         // Calculate how many bytes we want to copy from this frame:
         let copyLength = endOffsetInFrameInclusive - startOffsetInFrame + 1;

         // Calculate the physical addresses:
         let physStartAddress = physAddress.start_address() + startOffsetInFrame;

         // These are the offsets from the start address.
         // They correspond to the destination indices in `buffer`.
         let startOffsetInBuffer = Step::steps_between(&address, &startCopyAddress).unwrap();

         // Calculate the source slice utilising that frames are identity mapped:
         let source = unsafe {
            // SAFETY: We know that this memory is valid because we got it
            // as a result from a translation. There are not other
            // references to it.
            &*core::ptr::slice_from_raw_parts(physStartAddress.as_u64() as *const u8, copyLength)
         };

         // Calculate the destination pointer:
         let destination = &mut buffer[startOffsetInBuffer..][..copyLength];
         // Do the actual copy:
         destination.copy_from_slice(source);
      }
   }

   /// Write to the kernel address space.
   ///
   /// ## Safety
   /// - `address` should refer to a page mapped by a Load segment.
   ///
   /// ## Panics
   ///
   /// Panics if a page is not mapped in `self.pageTable`.
   unsafe fn copyTo(&mut self, address: VirtAddr, buffer: &mut [u8]) {
      // We can't know for sure that contiguous virtual address are contiguous
      // in physical memory, so we iterate of the pages spanning the
      // addresses, translate them to frames and copy the data.

      let endInclusiveAddress = Step::forward_checked(address, buffer.len() - 1)
         .expect("the end address should be within the virtual address space");

      let startPage = Page::<Size4KiB>::containing_address(endInclusiveAddress);
      let endInclusivePage = Page::<Size4KiB>::containing_address(endInclusiveAddress);

      for page in startPage..=endInclusivePage {
         // Translate the virtual page to the physical frame.
         let physAddress = unsafe {
            // SAFETY: The caller asserts that the pages are mapped by a Load segment.
            self.MakeMut(page)
         };

         // Figure out which addresses within the range we wish to copy to

         // This page covers these addresses:
         let pageStart = page.start_address();
         let pageEndInclusive = page.start_address() + 4095u64;

         // We wish to copy from the following addresses within this frame:
         let startCopyAddress = cmp::max(address, pageStart);
         let endInclusiveCopyAddress = cmp::min(endInclusiveAddress, pageEndInclusive);

         // These are the offsets into the frame we wish to copy from:
         let startOffsetInFrame = (startCopyAddress - pageStart) as usize;
         let endInclusiveOffsetInFrame = (endInclusiveCopyAddress - pageStart) as usize;

         // Calculate how many bytes we wish to copy from this frame.
         let copyLength = endInclusiveOffsetInFrame - startOffsetInFrame + 1;

         // Calculate the physical addresses:
         let startPhysAddress = physAddress.start_address() + startOffsetInFrame;

         // These are the offsets from the start address.
         // They correspond to the indices in `buffer`.
         let startOffsetInBuffer = Step::steps_between(&address, &startCopyAddress).unwrap();

         // Calculate the target slice utilising that frames are identity mapped:
         let destination = unsafe {
            // SAFETY: We know that this memory is valid because we got it
            // as a result from a translation. There are not other
            // references to it.
            &mut *core::ptr::slice_from_raw_parts_mut(
               startPhysAddress.as_u64() as *mut u8,
               copyLength,
            )
         };

         let source = &buffer[startOffsetInBuffer..][..copyLength];

         destination.copy_from_slice(source);
      }
   }
}

// IMPORTS //

use {
   crate::{api::TlsTemplate, level4::UsedLevel4Entries, Kernel, PAGE_SIZE},
   base::memory::VirtualAddressOffset,
   core::{cmp, iter::Step, mem::size_of, ops::Add},
   x86_64::{
      align_up,
      structures::paging::{
         mapper::{MappedFrame, MapperAllSizes, TranslateResult},
         FrameAllocator, Page, PageSize, PageTableFlags as Flags, PhysFrame, Size4KiB, Translate,
      },
      PhysAddr, VirtAddr,
   },
   xmas_elf::{
      dynamic, header,
      program::{self, ProgramHeader, SegmentData, Type},
      sections::Rela,
      ElfFile,
   },
};
