/// Abstraction trait for a memory region returned by the UEFI or BIOS firmware.
pub trait LegacyMemoryRegion: Copy + core::fmt::Debug {
   /// Return the physical start address of this region.
   fn start(&self) -> PhysAddr;

   /// Return the size of the region in bytes.
   fn length(&self) -> u64;

   /// Check whether this region is empty.
   fn isEmpty(&self) -> bool {
      self.length() == 0
   }

   /// The type of the region, e.g. whether or not it is usable or reserved elsewhere.
   fn kind(&self) -> MemoryRegionKind;

   /// Some regions become usable when the bootloader jumps to the kernel.
   fn usableAfterLoaderExit(&self) -> bool;
}

/// A physical frame allocator based on a BIOS or UEFI provided memory map.
pub struct LegacyFrameAllocator<I, D> {
   original: I,
   memoryMap: I,
   currentDescriptor: Option<D>,
   nextFrame: PhysFrame,
}

impl<I, D> LegacyFrameAllocator<I, D>
where
   I: ExactSizeIterator<Item = D> + Clone,
   I::Item: LegacyMemoryRegion,
{
   /// Creates a new frame allocator based on the given legacy memory regions.
   ///
   /// Skips the frame at physical address zero to avoid potential problems. For example
   /// identity-mapping the frame at address zero is not valid in Rust, because Rust's `core`
   /// library assumes that references can never point to virtual address `0`.  
   pub fn new(map: I) -> Self {
      let startFrame = PhysFrame::containing_address(PhysAddr::new(0x1000));
      return Self::newStartingAt(startFrame, map);
   }

   /// Creates a new frame allocator based on the given legacy memory regions. Skips any frames
   /// before the given `frame`.
   pub fn newStartingAt(frame: PhysFrame, map: I) -> Self {
      return Self {
         original: map.clone(),
         memoryMap: map,
         currentDescriptor: None,
         nextFrame: frame,
      };
   }

   fn allocateFrameFromDescriptor(&mut self, descriptor: D) -> Option<PhysFrame> {
      let startAddress = descriptor.start();
      let startFrame = PhysFrame::containing_address(startAddress);
      let endAddress = startAddress + descriptor.length();
      let endFrame = PhysFrame::containing_address(endAddress - 1u64);

      if self.nextFrame < startFrame {
         self.nextFrame = startFrame;
      }

      if self.nextFrame <= endFrame {
         let ret = self.nextFrame;
         self.nextFrame += 1;
         Some(ret)
      } else {
         None
      }
   }

   /// Returns the number of memory regions in the underlying memory map.
   ///
   /// The function always returns the same value, i.e. the length doesn't
   /// change after calls to `allocate_frame`.
   pub fn length(&self) -> usize {
      self.original.len()
   }

   /// Returns whether this memory map is empty.
   pub fn isEmpty(&self) -> bool {
      self.length() == 0
   }

   /// Returns the largest detected physical memory address.
   ///
   /// Useful for creating a mapping for all physical memory.
   pub fn maxPhysAddress(&self) -> PhysAddr {
      self
         .original
         .clone()
         .map(|r| r.start() + r.length())
         .max()
         .unwrap()
   }

   /// Converts this type to a boot info memory map.
   ///
   /// The memory map is placed in the given `regions` slice. The length of the given slice
   /// must be at least the value returned by [`len`] plus 1.
   ///
   /// The return slice is a subslice of `regions`, shortened to the actual number of regions.
   pub fn ConstructMemoryMap(
      self,
      regions: MaybeUninit<MemoryRegion>,
      kernelSliceStart: PhysAddr,
      kernelSliceLength: u64,
   ) -> &mut [MemoryRegion] {
      let mut nextIndex = 0;
      let kernelSliceStart = kernelSliceStart.as_u64();

      for descriptor in self.original {
         let mut start = descriptor.start();
         let end = start + descriptor.length();
         let nextFree = self.nextFrame.start_address();
         let kind = match descriptor.kind() {
            MemoryRegionKind::Usable => {
               if end <= nextFree {
                  MemoryRegionKind::Bootloader
               } else if descriptor.start() >= nextFree {
                  MemoryRegionKind::Usable
               } else {
                  let usedRegion = MemoryRegion {
                     start: descriptor.start().as_u64(),
                     end: nextFree.as_u64(),
                     kind: MemoryRegionKind::Bootloader,
                  };

                  Self::addRegion(usedRegion, regions, &mut nextIndex);

                  start = nextFree;
                  MemoryRegionKind::Usable
               }
            }

            _ if descriptor.usableAfterLoaderExit() => {
               // Region was not usable before, but it will be as soon as
               // the bootloader passes control to the kernel. We don't
               // need to check against `next_free` because the
               // LegacyFrameAllocator only allocates memory from usable
               // descriptors.
               MemoryRegionKind::Usable
            }

            other => other,
         };

         let region = MemoryRegion {
            start: start.as_u64(),
            end: end.as_u64(),
            kind,
         };

         // Check if the memory region overlaps with our kernel.
         let kernelSliceEnd = kernelSliceStart + kernelSliceLength;
         if region.kind == MemoryRegionKind::Usable
            && kernelSliceStart < region.end
            && kernelSliceEnd > region.start
         {
            // Region overlaps with kernel, so we may need to split it.

            // Ensure that the kernel allocation does not span multiple regions.
            assert!(
               kernelSliceStart >= region.start,
               "region overlaps with kernel, but kernel begins before region \
               (kernelSliceStart: {kernelSliceStart: #x}, regionStart: {:#x})",
               region.start
            );

            assert!(
               kernelSliceEnd <= region.end,
               "region overlaps with kernel, but region ends before kernel \
               (kernelSliceEnd: {kernelSliceEnd: #x}, regionEnd: {:#x})",
               region.end
            );

            let beforeKernel = MemoryRegion {
               end: kernelSliceStart,
               ..region
            };

            let kernel = MemoryRegion {
               start: kernelSliceStart,
               end: kernelSliceEnd,
               kind: MemoryRegionKind::Bootloader,
            };

            let afterKernel = MemoryRegion {
               start: kernelSliceEnd,
               ..region
            };

            // add the three regions (empty regions are ignored in `add_region`)
            Self::addRegion(beforeKernel, regions, &mut nextIndex);
            Self::addRegion(kernel, regions, &mut nextIndex);
            Self::addRegion(afterKernel, regions, &mut nextIndex);
         } else {
            // add the region normally
            Self::add_region(region, regions, &mut next_index);
         }
      }

      let initialised = &mut regions[..nextIndex];
      unsafe {
         // inlined variant of: `MaybeUninit::slice_assume_init_mut(initialized)`
         // TODO: undo inlining when `slice_assume_init_mut` becomes stable
         &mut *(initialized as *mut [_] as *mut [_])
      }
   }

   fn addRegion(
      region: MemoryRegion,
      regions: &mut [MaybeUninit<MemoryRegion>],
      nextIndex: &mut usize,
   ) {
      if region.start == region.end {
         // Skip zero-sized regions.
         return;
      }

      unsafe {
         regions
            .get_mut(*nextIndex)
            .expect("cannot add regions: there are no free entries in the memory map")
            .as_mut_ptr()
            .write(region)
      };

      *nextIndex += 1;
   }
}

unsafe impl<I, D> FrameAllocator<Size4KiB> for LegacyFrameAllocator<I, D>
   where
      I: ExactSizeIterator<Item = D> + Clone,
      I::Item: LegacyMemoryRegion,
{
   fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
      if let Some(current) = self.currentDescriptor {
         match self.allocateFrameFromDescriptor(current) {
            Some(frame) => return Some(frame),
            None => {
               self.currentDescriptor = None;
            }
         }
      }

      while let Some(descriptor) = self.memoryMap.next() {
         if descriptor.kind() != MemoryRegionKind::Usable {
            continue;
         }

         if let Some(frame) = self.allocateFrameFromDescriptor(descriptor) {
            self.currentDescriptor = Some(descriptor);
            return Some(frame);
         }
      }

      return None;
   }
}

// IMPORTS //

use {
   crate::api::info::{MemoryRegion, MemoryRegionKind},
   core::mem::MaybeUninit,
   x86_64::{
      structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
      PhysAddr,
   },
};
