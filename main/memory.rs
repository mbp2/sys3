pub unsafe fn initialise(physical_offset: VirtAddr) -> OffsetPageTable<'static> {
   let l4table = active_l4_page_table(physical_offset);

   log::info!("Got the level four page table.");

   return OffsetPageTable::new(l4table, physical_offset);
}

pub fn build_heap(
   mapper: &mut impl Mapper<Size4KiB>,
   frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
   let mut heap = HEAP.lock();
   let start_address = HEAP_START;
   let end_address = HEAP_START + HEAP_SIZE - 1usize;

   let page_range = {
      let heap_start_page = Page::containing_address(VirtAddr::new(start_address as u64));
      let heap_end_page = Page::containing_address(VirtAddr::new(end_address as u64));
      Page::range_inclusive(heap_start_page, heap_end_page)
   };

   for page in page_range {
      let frame = frame_allocator
         .allocate_frame()
         .ok_or(MapToError::FrameAllocationFailed)?;

      let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
      unsafe {
         mapper.map_to(page, frame, flags, frame_allocator)?.flush();
      }
   }

   unsafe {
      let mut blocks = Heap::new();
      blocks.add_to_heap(start_address, end_address);

      *heap = Some(blocks);
   }

   log::info!("Successfully initialised system heap.");

   return Ok(());
}

pub unsafe fn active_l4_page_table(phys_offset: VirtAddr) -> &'static mut PageTable {
   use x86_64::registers::control::Cr3;

   let (l4Frame, _) = Cr3::read();

   let physicalAddress = l4Frame.start_address();
   let virtualAddress = phys_offset + physicalAddress.as_u64();
   let pageTablePointer: *mut PageTable = virtualAddress.as_mut_ptr();

   return &mut *pageTablePointer; // This dereference is unsafe.
}

/// Translates the given virtual address to the mapped physical address, or
/// `None` if the address is not mapped.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`.
pub fn translate_address(address: VirtAddr, physical_offset: VirtAddr) -> Option<PhysAddr> {
   use x86_64::structures::paging::page_table::FrameError;
   use x86_64::registers::control::Cr3;

   let (l4Frame, _) = Cr3::read();

   let tableIndices = [
      address.p4_index(),
      address.p3_index(),
      address.p2_index(),
      address.p1_index(),
   ];

   let mut frame = l4Frame;

   // Traverse the multi-level page table.
   for &index in &tableIndices {
      // convert the frame into a page table reference
      let virt = physical_offset + frame.start_address().as_u64();
      let table_pointer: *const PageTable = virt.as_ptr();
      let table = unsafe {&*table_pointer};

      // read the page table entry and update `frame`
      let entry = &table[index];
      frame = match entry.frame() {
         Ok(frame) => frame,
         Err(FrameError::FrameNotPresent) => return None,
         Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
      };
   }

   return Some(frame.start_address() + u64::from(address.page_offset()));
}

pub struct SystemFrameAllocator {
   memory_map: &'static [MemoryRegion],
   next: usize,
}

impl SystemFrameAllocator {
   pub unsafe fn new(memory_map: &'static [MemoryRegion]) -> Self {
      return SystemFrameAllocator{
         memory_map,
         next: 0,
      };
   }

   pub fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
      let regions = self.memory_map.iter();
      let usable = regions
         .filter(|r| r.kind == MemoryRegionKind::Usable);

      let addressRanges = usable.map(|r| r.start..r.end);
      let frameAddresses = addressRanges.flat_map(|r| r.step_by(4096));
      return frameAddresses.map(|address| PhysFrame::containing_address(PhysAddr::new(address)));
   }
}

unsafe impl FrameAllocator<Size4KiB> for SystemFrameAllocator {
   fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
      let frame = self.usable_frames().nth(self.next);
      self.next += 1;
      return frame;
   }
}

// IMPORTS //

use {
   base::{alloc::heap::{HEAP, Heap, HEAP_SIZE, HEAP_START}, log},
   springboard_api::info::{
      MemoryRegion, MemoryRegionKind,
   },
   x86_64::{
      structures::paging::{
         FrameAllocator,
         PageTable,
         PhysFrame,
         Size4KiB,
         Mapper,
         OffsetPageTable,
         Page,
         PageTableFlags,
         mapper::MapToError,
      },
      PhysAddr, VirtAddr,
   },
};
