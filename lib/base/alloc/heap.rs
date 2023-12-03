/// Either our global system heap, or `None` if it hasn't yet been allocated.
pub static HEAP: Mutex<Option<Heap<'static>>> = Mutex::new(None);

pub unsafe fn build_heap(
   heapBase: *mut u8,
   heapSize: usize,
   freeLists: &'static mut [*mut FreeBlock],
) {
   let mut heap = HEAP.lock();
   *heap = Some(Heap::new(heapBase, heapSize, freeLists));
}

const MIN_HEAP_ALIGN: usize = 4096;

/// A free block in our heap.  This is actually a header that we store at
/// the start of the block.  We don't store any size information in the
/// header, because we a separate free block array for each block size.
pub struct FreeBlock {
   /// The next available free block or "null" if it is the final block.
   next: *mut FreeBlock,
}

impl FreeBlock {
   /// Construct a `FreeBlock` header pointing at `next`.
   pub fn new(next: *mut FreeBlock) -> Self {
      FreeBlock { next }
   }
}

/// The interface to a heap.
///
/// This data structure is stored _outside_ the
/// heap somewhere, because every single byte of our heap is potentially
/// available for allocation.
pub struct Heap<'a> {
   /// The base address of our heap.
   ///
   /// Must be aligned along the boundary of `MIN_HEAP_ALIGN`.
   heapBase: *mut u8,

   /// The size of our heap.
   ///
   /// This must be a power of two.
   heapSize: usize,

   /// The free lists for our heap.
   ///
   /// The block at `free_lists[0]` contains the smallest block that we can allocate,
   /// and the array at the end can only contain a single free block the size of our entire heap,
   /// and only when no memory is allocated.
   freeLists: &'a mut [*mut FreeBlock],

   /// Our minimum block size.
   ///
   /// This is calculated based on `heap_size`
   /// and the length of the provided `free_lists` array, and it must be
   /// big enough to contain a `FreeBlock` header object.
   minBlockSize: usize,

   /// The log base 2 of our block size.
   ///
   /// Cached here so we don't have to
   /// recompute it on every allocation (but we haven't benchmarked the
   /// performance gain).
   minBlockSizeLog2: u8,
}

unsafe impl<'a> Send for Heap<'a> {}

impl<'a> Heap<'a> {
   pub unsafe fn new(heapBase: *mut u8, heapSize: usize, freeLists: &mut [*mut FreeBlock]) -> Heap {
      // The heap base must not be null.
      assert_ne!(heapBase, ptr::null_mut());

      // We must have at least one free array.
      assert!(freeLists.len() > 0);

      // Calculate our minimum block size based on the number of free
      // lists we have available.
      let minBlockSize = heapSize >> (freeLists.len() - 1);

      // The heap must be aligned on a 4K boundary.
      assert_eq!(heapBase as usize & (MIN_HEAP_ALIGN - 1), 0);

      // The heap must be big enough to contain at least one block.
      assert!(heapSize >= minBlockSize);

      // The smallest possible heap block must be big enough to contain
      // the block header.
      assert!(minBlockSize >= size_of::<FreeBlock>());

      // The heap size must be a power of 2.  See:
      // http://graphics.stanford.edu/~seander/bithacks.html#DetermineIfPowerOf2
      assert!(heapSize.powerOf2());

      // We must have one free array per possible heap block size.
      assert_eq!(
         minBlockSize * (2u32.pow(freeLists.len() as u32 - 1)) as usize,
         heapSize
      );

      // Zero out our free array pointers.
      for pointer in freeLists.iter_mut() {
         *pointer = ptr::null_mut();
      }

      // Store all the info about our heap in our struct.
      let mut result = Heap {
         heapBase,
         heapSize,
         freeLists,
         minBlockSize,
         minBlockSizeLog2: minBlockSize.log2(),
      };

      // Insert the entire heap onto the appropriate free array as a
      // single block.
      let order = result
         .allocationOrder(heapSize, 1)
         .expect("Failed to calculate order for root heap block");

      result.freeListInsert(order, heapBase);

      // Return our newly-created heap.
      return result;
   }

   /// Figure out what size block we'll need to fulfill an allocation
   /// request.  This is deterministic, and it does not depend on what
   /// we've already allocated.  In particular, it's important to be able
   /// to calculate the same `allocation_size` when freeing memory as we
   /// did when allocating it, or everything will break horribly.
   pub fn allocationSize(&self, mut size: usize, align: usize) -> Option<usize> {
      // Sorry, we don't support weird alignments.
      if !align.powerOf2() {
         return None;
      }

      // We can't align any more precisely than our heap base alignment
      // without getting much too clever, so don't bother.
      if align > MIN_HEAP_ALIGN {
         return None;
      }

      // We're automatically aligned to `size` because of how our heap is
      // sub-divided, but if we need a larger alignment, we can only do
      // it be allocating more memory.
      if align > size {
         size = align;
      }

      // We can't allocate blocks smaller than `min_block_size`.
      size = max(size, self.minBlockSize);

      // Round up to the next power of two.
      size = size.nextPowerOf2();

      // We can't allocate a block bigger than our heap.
      if size > self.heapSize {
         return None;
      }

      Some(size)
   }

   /// The "order" of an allocation is how many times we need to double
   /// `min_block_size` in order to get a large enough block, as well as
   /// the index we use into `free_lists`.
   pub fn allocationOrder(&self, size: usize, align: usize) -> Option<usize> {
      self
         .allocationSize(size, align)
         .map(|s| (s.log2() - self.minBlockSizeLog2) as usize)
   }

   /// The size of the blocks we allocate for a given order.
   fn orderSize(&self, order: usize) -> usize {
      1 << (self.minBlockSizeLog2 as usize + order)
   }

   /// Pop a block off the appropriate free array.
   unsafe fn freeListPop(&mut self, order: usize) -> Option<*mut u8> {
      let candidate = self.freeLists[order];
      return if candidate != ptr::null_mut() {
         self.freeLists[order] = (*candidate).next;
         Some(candidate as *mut u8)
      } else {
         None
      };
   }

   /// Insert `block` of order `order` onto the appropriate free array.
   unsafe fn freeListInsert(&mut self, order: usize, block: *mut u8) {
      let freePointer = block as *mut FreeBlock;
      *freePointer = FreeBlock::new(self.freeLists[order]);
      self.freeLists[order] = freePointer;
   }

   /// Attempt to remove a block from our free array, returning true
   /// success, and false if the block wasn't on our free array.  This is
   /// the slowest part of a primitive buddy allocator, because it runs in
   /// O(log N) time where N is the number of blocks of a given size.
   ///
   /// We could perhaps improve this by keeping our free lists sorted,
   /// because then "nursery generation" allocations would probably tend
   /// to occur at lower addresses and then be faster to find / rule out
   /// finding.
   unsafe fn freeListRemove(&mut self, order: usize, block: *mut u8) -> bool {
      let blockPointer = block as *mut FreeBlock;

      // Yuck, array traversals are gross without recursion.  Here,
      // `*checking` is the pointer we want to check, and `checking` is
      // the memory location we found it at, which we'll need if we want
      // to replace the value `*checking` with a new value.
      let mut checking: *mut *mut FreeBlock = &mut self.freeLists[order];

      // Loop until we run out of free blocks.
      while *checking != ptr::null_mut() {
         // Is this the pointer we want to remove from the free array?
         if *checking == blockPointer {
            // Yup, this is the one, so overwrite the value we used to
            // get here with the next one in the sequence.
            *checking = (*(*checking)).next;
            return true;
         }

         // Haven't found it yet, so point `checking` at the address
         // containing our `next` field.  (Once again, this is so we'll
         // be able to reach back and overwrite it later if necessary.)
         checking = &mut ((*(*checking)).next);
      }

      return false;
   }

   /// Split a `block` of order `order` down into a block of order
   /// `order_needed`, placing any unused chunks on the free array.
   unsafe fn splitFreeBlock(&mut self, block: *mut u8, mut order: usize, order_needed: usize) {
      // Get the size of our starting block.
      let mut size_to_split = self.orderSize(order);

      // Progressively cut our block down to size.
      while order > order_needed {
         // Update our loop counters to describe a block half the size.
         size_to_split >>= 1;
         order -= 1;

         // Insert the "upper half" of the block into the free array.
         let split = block.offset(size_to_split as isize);
         self.freeListInsert(order, split);
      }
   }

   /// Allocate a block of memory large enough to contain `size` bytes,
   /// and aligned on `align`.  This will return NULL if the `align` is
   /// greater than `MIN_HEAP_ALIGN`, if `align` is not a power of 2, or
   /// if we can't find enough memory.
   ///
   /// All allocated memory must be passed to `deallocate` with the same
   /// `size` and `align` parameter, or else horrible things will happen.
   pub unsafe fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
      // Figure out which order block we need.
      if let Some(order_needed) = self.allocationOrder(size, align) {
         // Start with the smallest acceptable block size, and search
         // upwards until we reach blocks the size of the entire heap.
         for order in order_needed..self.freeLists.len() {
            // Do we have a block of this size?
            if let Some(block) = self.freeListPop(order) {
               // If the block is too big, break it up.  This leaves
               // the address unchanged, because we always allocate at
               // the head of a block.
               if order > order_needed {
                  self.splitFreeBlock(block, order, order_needed);
               }

               // We have an allocation, so quit now.
               return block;
            }
         }

         // We couldn't find a large enough block for this allocation.
         ptr::null_mut()
      } else {
         // We can't allocate a block with the specified size and
         // alignment.
         ptr::null_mut()
      }
   }

   /// Given a `block` with the specified `order`, find the "buddy" block,
   /// that is, the other half of the block we originally split it from,
   /// and also the block we could potentially merge it with.
   pub unsafe fn buddy(&self, order: usize, block: *mut u8) -> Option<*mut u8> {
      let relative = (block as usize) - (self.heapBase as usize);
      let size = self.orderSize(order);
      if size >= self.heapSize {
         // The main heap itself does not have a budy.
         None
      } else {
         // Fun: We can find our buddy by xoring the right bit in our
         // offset from the base of the heap.
         Some(self.heapBase.offset((relative ^ size) as isize))
      }
   }

   /// Deallocate a block allocated using `allocate`.  Note that the
   /// `old_size` and `align` values must match the values passed to
   /// `allocate`, or our heap will be corrupted.
   pub unsafe fn deallocate(&mut self, pointer: *mut u8, oldSize: usize, align: usize) {
      let initial_order = self
         .allocationOrder(oldSize, align)
         .expect("Tried to dispose of invalid block");

      // The fun part: When deallocating a block, we also want to check
      // to see if its "buddy" is on the free array.  If the buddy block
      // is also free, we merge them and continue walking up.
      //
      // `block` is the biggest merged block we have so far.
      let mut block = pointer;
      for order in initial_order..self.freeLists.len() {
         // Would this block have a buddy?
         if let Some(buddy) = self.buddy(order, block) {
            // Is this block's buddy free?
            if self.freeListRemove(order, buddy) {
               // Merge them!  The lower address of the two is the
               // newly-merged block.  Then we want to try again.
               block = min(block, buddy);
               continue;
            }
         }

         // If we reach here, we didn't find a buddy block of this size,
         // so take what we've got and mark it as free.
         self.freeListInsert(order, block);
         return;
      }
   }
}

// IMPORTS //

use {
   crate::math::PowersOf2,
   core::{
      cmp::{max, min},
      mem::size_of,
      ptr,
   },
   spin::Mutex,
};
