pub static HEAP: Mutex<Option<Heap<32>>> = Mutex::new(None);

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub struct Heap<const ORDER: usize> {
   pub allocated: usize,
   pub freeList: [LinkedList; ORDER],
   pub total: usize,
   pub user: usize,
}

impl<const ORDER: usize> Heap<ORDER> {
   /// Create an empty heap.
   pub const fn new() -> Self {
      return Heap{
         allocated: 0,
         freeList: [LinkedList::new(); ORDER],
         total: 0,
         user: 0,
      };
   }

   pub unsafe fn add_to_heap(&mut self, mut start: usize, mut end: usize) {
      // Avoid unaligned access.
      start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
      end &= !size_of::<usize>() + 1;
      assert!(start <= end);

      let mut total = 0;
      let mut current_start = start;

      while current_start + size_of::<usize>() <= end {
         let lowbit = current_start & (!current_start + 1);
         let size = min(lowbit, previous_po2(end - current_start));
         total += size;

         self.freeList[size.trailing_zeros() as usize].push(current_start as *mut usize);
         current_start += size;
      }

      self.total += total;
   }

   /// Allocate a block of memory large enough to contain `size` bytes,
   /// and aligned on `align`.  This will return NULL if the `align` is
   /// greater than `MIN_HEAP_ALIGN`, if `align` is not a power of 2, or
   /// if we can't find enough memory.
   ///
   /// All allocated memory must be passed to `deallocate` with the same
   /// `size` and `align` parameter, or else horrible things will happen.
   pub unsafe fn allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocationError> {
      let size = max(
         layout.size.nextPowerOf2(),
         max(layout.align, size_of::<usize>()),
      );

      let class = size.trailing_zeros() as usize;
      for i in class..self.freeList.len() {
         // Find the first non-empty size class
         if !self.freeList[i].empty() {
            // Split the buffers.
            for j in (class + 1..i + 1).rev() {
               if let Some(block) = self.freeList[j].pop() {
                  unsafe {
                     self.freeList[j - 1].push((block as usize + (1 << (j - 1))) as *mut usize);
                     self.freeList[j - 1].push(block);
                  }
               } else {
                  return Err(AllocationError);
               }
            }

            let result = NonNull::new(
               self.freeList[class].pop()
                  .expect("current block should have free space") as *mut u8
            );

            return if let Some(result) = result {
               self.user += layout.size;
               self.allocated += size;
               Ok(result)
            } else {
               Err(AllocationError)
            };
         }
      }

      return Err(AllocationError);
   }

   pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
      let size = max(
         layout.size.nextPowerOf2(),
         max(layout.align, size_of::<usize>()),
      );

      let class = size.trailing_zeros() as usize;

      // Place the block back into the free list.
      self.freeList[class].push(ptr.as_ptr() as *mut usize);

      // Merge buddy free lists.
      let mut currentPointer = ptr.as_ptr() as usize;
      let mut currentClass = class;

      while currentClass < self.freeList.len() {
         let buddy = currentPointer ^ (1 << currentClass);
         let mut flag = false;

         for block in self.freeList[currentClass].iterator_mut() {
            if block.value() as usize == buddy {
               block.pop();
               flag = true;
               break;
            }
         }

         if flag {
            self.freeList[currentClass].pop();
            currentPointer = min(currentPointer, buddy);
            currentClass += 1;
            self.freeList[currentClass].push(currentPointer as *mut usize);
         } else {
            break;
         }
      }

      self.user -= layout.size;
      self.allocated -= size;
   }
}

impl<const ORDER: usize> Debug for Heap<ORDER> {
   fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
      f.debug_struct("Heap")
         .field("user", &self.user)
         .field("allocated", &self.allocated)
         .field("total", &self.total)
         .finish()
   }
}

/// A locked version of `Heap`
///
/// # Usage
///
/// Create a locked heap and add a memory region to it:
/// ```
/// use base::alloc::heap::*;
/// # use core::mem::size_of;
/// let mut heap = LockedHeap::<32>::new();
/// # let space: [usize; 100] = [0; 100];
/// # let begin: usize = space.as_ptr() as usize;
/// # let end: usize = begin + 100 * size_of::<usize>();
/// # let size: usize = 100 * size_of::<usize>();
/// unsafe {
///     heap.lock().init(begin, size);
///     // or
///     heap.lock().add_to_heap(begin, end);
/// }
/// ```
pub struct LockedHeap<const ORDER: usize>(Mutex<Heap<ORDER>>);

impl<const ORDER: usize> LockedHeap<ORDER> {
   pub const fn new() -> Self {
      return LockedHeap(Mutex::new(Heap::<ORDER>::new()));
   }
}

unsafe impl<const ORDER: usize> GlobalAlloc for LockedHeap<ORDER> {
   unsafe fn alloc(&self, layout: StdLayout) -> *mut u8 {
      let layout = Layout::from(layout);

      self.0.lock()
         .allocate(layout)
         .ok()
         .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
   }

   unsafe fn dealloc(&self, ptr: *mut u8, layout: StdLayout) {
      let layout = Layout::from(layout);
      self.0.lock().deallocate(NonNull::new_unchecked(ptr), layout);
   }
}

// IMPORTS //

use {
   crate::{
      alloc::{AllocationError, Layout},
      array::linked_list::LinkedList,
      math::{previous_po2, PowersOf2},
   },
   std_alloc::alloc::{GlobalAlloc, Layout as StdLayout},
   core::{
      cmp::{min, max},
      fmt::{Debug, Formatter, Result as FmtResult},
      mem::size_of,
      ops::Deref,
      ptr::NonNull,
   },
   spin::Mutex,
   x86_64::{
      structures::paging::{
         mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
      },
      VirtAddr,
   },
};
