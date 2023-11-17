/// Keeps track of used entries in a level 4 page table.
///
/// Useful for determining a free virtual memory block, e.g. for mapping additional data.
pub struct UsedLevel4Entries {
   /// Whether an entry is in use by the kernel.
   entryState: [bool; 512],
   /// A random number generator that should be used to generate random addresses or
   /// `None` if aslr is disabled.
   rng: Option<Hc128Rng>,
}

impl UsedLevel4Entries {
   /// Initializes a new instance.
   ///
   /// Marks the statically configured virtual address ranges from the config as used.
   pub fn new(
      maxPhysAddress: PhysAddr,
      regionsLength: usize,
      framebuffer: Option<&RawPixelBufferInfo>,
      config: &BootloaderConfig,
   ) -> Self {
      let mut used = UsedLevel4Entries {
         entryState: [false; 512],
         rng: config.mappings.aslr.then(entropy::BuildRng),
      };

      used.entryState[0] = true; // TODO: Can this be done dynamically?

      // Mark the statically configured ranges from the configuration we used.

      if let Some(Mapping::Fixed(physMemoryOffset)) = config.mappings.physicalMemory {
         used.markRange(physMemoryOffset, maxPhysAddress.as_u64().into_usize());
      }

      if let Some(Mapping::Fixed(recursive)) = config.mappings.pageRecursiveTable {
         let recursiveIndex = VirtAddr::new(recursive).p4_index();
         used.markP4Index(recursiveIndex);
      }

      if let Mapping::Fixed(kernelStackAddress) = config.mappings.kernelStack {
         used.markRange(kernelStackAddress, config.kernelStackSize);
      }

      if let Mapping::Fixed(bootInfoAddress) = config.mappings.bootInfo {
         let bootInfoLayout = Layout::new::<BootInfo>();
         let regions = regionsLength + 1; // One region may be split into used/unused.
         let memRegionsLayout = Layout::array::<MemoryRegion>(regions).unwrap();
         let (combined, _) = bootInfoLayout.extend(memRegionsLayout).unwrap();

         used.markRange(bootInfoAddress, combined.size())
      }

      if let Mapping::Fixed(fbAddress) = config.mappings.framebuffer {
         if let Some(framebuffer) = framebuffer {
            used.markRange(fbAddress, framebuffer.info.byteLength);
         }
      }

      // Mark everything before the dynamic range unusable.
      if let Some(dynRangeStart) = config.mappings.dynamicRangeStart {
         let dynRangeStart = VirtAddr::new(dynRangeStart);
         let startPage: Page = Page::containing_address(dynRangeStart);
         if let Some(unusablePage) = Step::backward_checked(startPage, 1) {
            for index in 0..=u16::from(unusablePage.p4_index()) {
               used.markP4Index(PageTableIndex::new(index));
            }
         }
      }

      if let Some(dynRangeEnd) = config.mappings.dynamicRangeEnd {
         let dynRangeEnd = VirtAddr::new(dynRangeEnd);
         let endPage: Page = Page::containing_address(dynRangeEnd);
         if let Some(unusablePage) = Step::forward_checked(endPage, 1) {
            for index in u16::from(unusablePage.p4_index())..512 {
               used.markP4Index(PageTableIndex::new(index));
            }
         }
      }

      used
   }

   /// Marks all p4 entries in the range `[address..address+size)` as used.
   ///
   /// `size` can be a `u64` or `usize`.
   fn markRange<S>(&mut self, address: u64, size: S)
   where
      VirtAddr: core::ops::Add<S, Output = VirtAddr>, {
      let start = VirtAddr::new(address);
      let endInclusive = (start + size) - 1usize;
      let startPage = Page::<Size4KiB>::containing_address(start);
      let endPageInclusive = Page::<Size4KiB>::containing_address(endInclusive);

      for index in u16::from(startPage.p4_index())..=u16::from(endPageInclusive.p4_index()) {
         self.markP4Index(PageTableIndex::new(index));
      }
   }

   fn markP4Index(&mut self, index: PageTableIndex) {
      self.entryState[usize::from(index)] = true;
   }

   /// Marks the virtual address range of all segments as used.
   pub fn MarkSegments<'a>(
      &mut self,
      segments: impl Iterator<Item = ProgramHeader<'a>>,
      offset: VirtualAddressOffset,
   ) {
      for segment in segments.filter(|s| s.mem_size() > 0) {
         self.markRange(offset + segment.virtual_addr(), segment.mem_size());
      }
   }

   /// Returns the first index of a `num` contiguous unused level 4 entries and marks them as
   /// used. If `CONFIG.aslr` is enabled, this will return random contiguous available entries.
   ///
   /// Since this method marks each returned index as used, it can be used multiple times
   /// to determine multiple unused virtual memory regions.
   pub fn GetFreeEntries(&mut self, num: u64) -> PageTableIndex {
      // Create an iterator over all available p4 indices with `num` contiguous free entries.
      let mut freeEntries = self
         .entryState
         .windows(num.into_usize())
         .enumerate()
         .filter(|(_, entries)| entries.iter().all(|used| !used))
         .map(|(index, _)| index);

      let indexOpt = if let Some(rng) = self.rng.as_mut() {
         freeEntries.choose(rng)
      } else {
         freeEntries.next()
      };

      let Some(index) = indexOpt else {
         panic!("no usable level 4 entries found ({num} entries requested)");
      };

      for i in 0..num.into_usize() {
         self.entryState[index + i] = true;
      }

      return PageTableIndex::new(index.try_into().unwrap());
   }

   /// Returns a virtual address in one or more unused level 4 entries and marks them as used.
   ///
   /// This function calls [`GetFreeEntries`] internally, so all of its docs applies here
   /// too.
   pub fn GetFreeAddress(&mut self, size: u64, alignment: u64) -> VirtAddr {
      assert!(alignment.is_power_of_two());

      const LEVEL4_SIZE: u64 = 4096 * 512 * 512 * 512;

      let level4Entries = (size + (LEVEL4_SIZE - 1)) / LEVEL4_SIZE;
      let base = Page::from_page_table_indices_1gib(
         self.GetFreeEntries(level4Entries),
         PageTableIndex::new(0),
      )
      .start_address();

      let offset = if let Some(rng) = self.rng.as_mut() {
         // Choose a random offset
         let maxOffset = LEVEL4_SIZE - (size % LEVEL4_SIZE);
         let uniformRange = Uniform::from(0..maxOffset / alignment);
         uniformRange.sample(rng) * alignment
      } else {
         0
      };

      return base + offset;
   }
}

// IMPORTS //

use {
   crate::{
      api::info::{BootInfo, MemoryRegion},
      config::{self, BootConfig, Mapping},
      entropy, BootloaderConfig, RawPixelBufferInfo,
   },
   base::memory::VirtualAddressOffset,
   core::{alloc::Layout, iter::Step},
   rand::{
      distributions::{Distribution, Uniform},
      seq::IteratorRandom,
   },
   rand_hc::Hc128Rng,
   usize_conversions::IntoUsize,
   x86_64::{
      structures::paging::{Page, PageTableIndex, Size4KiB},
      PhysAddr, VirtAddr,
   },
   xmas_elf::program::ProgramHeader,
};
