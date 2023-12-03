#[repr(usize)]
pub enum ListFlags {
   Taken = 1 << 63,
}

impl ListFlags {
   /// Return the value of `self` as a `usize`.
   pub fn value(self) -> usize {
      return self as usize;
   }
}

#[derive(Clone, Copy, Debug)]
pub struct AllocateList {
   pub flag_size: usize,
}

impl AllocateList {
   pub fn taken(&self) -> bool {
      return self.flag_size & ListFlags::Taken.value() != 0;
   }

   pub fn free(&self) -> bool {
      return !self.taken();
   }

   pub fn take(&mut self) {
      self.flag_size |= ListFlags::Taken.value();
   }

   pub fn set_free(&mut self) {
      self.flag_size &= !ListFlags::Taken.value();
   }

   pub fn set_size(&mut self, size: usize) {
      self.flag_size = size & !ListFlags::Taken.value();
      if self.taken() {
         self.flag_size |= ListFlags::Taken.value();
      }
   }

   pub fn size(&self) -> usize {
      return self.flag_size & !ListFlags::Taken.value();
   }
}

/// This is the head of the allocation.
/// We start here when we need to look for free memory.
pub static mut KERNEL_MEM_HEAD: *mut AllocateList = ptr::null_mut();

/// In the future, we will have on-demand pages so we need to
/// keep track of our memory to make sure we can actually
/// allocate more.
pub static mut KERNEL_MEM_ALLOCATIONS: usize = 0;

/// The page table for our reserved kernel memory.
pub static mut KERNEL_PAGE_TABLE: *mut PageTable = ptr::null_mut();

// GLOBAL ALLOCATOR //

/// Our global system allocator.
///
/// This structure has no members because it is simply a unit with which to access
/// globally-available memory allocation routines.
pub struct SystemAllocator;

/// Our exposed global kernel allocator.
pub static OSGA: SystemAllocator = SystemAllocator;

// IMPORTS //

use {
   //alloc::alloc::{GlobalAlloc, Layout},
   base::alloc::paging::{PAGE_SIZE},
   core::{
      mem::size_of,
      ptr
   },
   x86_64::structures::paging::PageTable,
};
