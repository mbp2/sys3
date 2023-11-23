#[doc(hidden)]
#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
   unsafe {
      HEAP
         .lock()
         .as_mut()
         .expect("must initialise heap before calling")
         .allocate(size, align)
   }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn __rust_deallocate(pointer: *mut u8, oldSize: usize, align: usize) {
   unsafe {
      HEAP
         .lock()
         .as_mut()
         .expect("must initialise heap before calling")
         .deallocate(pointer, oldSize, align)
   }
}

pub extern "C" fn __rust_reallocate(
   pointer: *mut u8,
   oldSize: usize,
   newSize: usize,
   align: usize,
) -> *mut u8 {
   let newPointer = __rust_allocate(newSize, align);
   return if newPointer.is_null() {
      newPointer
   } else {
      unsafe {
         ptr::copy(pointer, newPointer, min(newSize, oldSize));
      }

      __rust_deallocate(pointer, oldSize, align);
      newPointer
   };
}

/// We do not support in-place reallocation, so just return `oldSize`.
#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace(
   _pointer: *mut u8,
   oldSize: usize,
   _size: usize,
   _align: usize,
) -> usize {
   oldSize
}

/// I have no idea what this actually does, but we're supposed to have one,
/// and the other backends to implement it as something equivalent to the
/// following.
#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, _align: usize) -> usize {
   size
}

#[doc(hidden)]
pub unsafe fn alloc_one<T>(allocator: &mut dyn Allocator) -> Option<NonNull<T>> {
   allocator.allocate_aligned(Layout::new::<T>()).map(|ptr| ptr.cast::<T>())
}

#[doc(hidden)]
pub unsafe fn alloc_array<T>(allocator: &mut dyn Allocator, size: usize) -> Option<NonNull<T>> {
   allocator
      .allocate_aligned(Layout::from_type_array::<T>(size))
      .map(|ptr| ptr.cast::<T>())
}

#[derive(Debug)]
pub struct AllocationError;

impl Display for AllocationError {
   fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
      write!(f, "An error occurred allocating the requested memory")
   }
}

/// A global allocator for our program.
#[derive(Copy, Clone)]
pub struct GlobalAllocator;

/// A simple wrapper around `spin::Mutex` to enable trait implementations.
pub struct LockedAllocator<A: Allocator>(pub Mutex<A>);

impl<A> LockedAllocator<A>
where
   A: Allocator,
{
   pub const fn new(allocator: A) -> Self {
      return LockedAllocator(Mutex::new(allocator));
   }

   pub fn lock(&self) -> MutexGuard<A> {
      return self.0.lock();
   }
}

pub unsafe trait Allocator {
   fn allocate(&self, layout: Layout) -> Option<NonNull<u8>>;
   unsafe fn deallocate(&self, pointer: *mut u8, layout: Layout);
   unsafe fn reallocate(
      &self,
      pointer: *mut u8,
      oldSize: usize,
      layout: Layout,
   ) -> Option<NonNull<u8>>;

   unsafe fn allocate_aligned(&self, layout: Layout) -> Option<NonNull<u8>> {
      let actualSize = layout.size + layout.align - 1 + size_of::<usize>();

      let pointer = match self.allocate(Layout::from_size(actualSize)) {
         Some(p) => p.as_ptr() as usize,
         None => return None,
      };

      let alignedPointer = layout.align_up(pointer + size_of::<usize>());
      let actualP2P = alignedPointer - size_of::<usize>();

      ptr::write_unaligned(actualP2P as *mut usize, pointer);

      return Some(NonNull::new_unchecked(alignedPointer as *mut u8));
   }

   unsafe fn deallocate_aligned(&self, pointer: *mut u8, layout: Layout) {
      let alignedPointer = pointer as usize;
      let actualP2P = alignedPointer - size_of::<usize>();
      let actualPointer = ptr::read_unaligned(actualP2P as *const usize);

      self.deallocate(actualPointer as *mut u8, layout);
   }
}

unsafe impl<A: Allocator> Allocator for Mutex<A> {
   fn allocate(&self, layout: Layout) -> Option<NonNull<u8>> {
      return self.lock().allocate(layout);
   }

   unsafe fn deallocate(&self, pointer: *mut u8, layout: Layout) {
      self.lock().deallocate(pointer, layout);
   }

   unsafe fn reallocate(
      &self,
      pointer: *mut u8,
      oldSize: usize,
      layout: Layout,
   ) -> Option<NonNull<u8>> {
      return self.lock().reallocate(pointer, oldSize, layout);
   }
}

unsafe impl<A: Allocator> Allocator for LockedAllocator<A> {
   fn allocate(&self, layout: Layout) -> Option<NonNull<u8>> {
      return self.lock().allocate(layout);
   }

   unsafe fn deallocate(&self, pointer: *mut u8, layout: Layout) {
      return self.lock().deallocate(pointer, layout);
   }

   unsafe fn reallocate(
      &self,
      pointer: *mut u8,
      oldSize: usize,
      layout: Layout,
   ) -> Option<NonNull<u8>> {
      return self.lock().reallocate(pointer, oldSize, layout);
   }
}

unsafe impl<A: Allocator> Allocator for &RefCell<A> {
   fn allocate(&self, layout: Layout) -> Option<NonNull<u8>> {
      return self.borrow_mut().allocate(layout);
   }

   unsafe fn deallocate(&self, pointer: *mut u8, layout: Layout) {
      return self.borrow_mut().deallocate(pointer, layout);
   }

   unsafe fn reallocate(
      &self,
      pointer: *mut u8,
      oldSize: usize,
      layout: Layout,
   ) -> Option<NonNull<u8>> {
      return self.borrow_mut().reallocate(pointer, oldSize, layout);
   }
}

unsafe impl Allocator for GlobalAllocator {
   fn allocate(&self, layout: Layout) -> Option<NonNull<u8>> {
      return Some(NonNull::new(__rust_allocate(layout.size, layout.align)).unwrap());
   }

   unsafe fn deallocate(&self, pointer: *mut u8, layout: Layout) {
      __rust_deallocate(pointer, layout.size, layout.align);
   }

   unsafe fn reallocate(
      &self,
      pointer: *mut u8,
      oldSize: usize,
      layout: Layout,
   ) -> Option<NonNull<u8>> {
      return Some(
         NonNull::new(__rust_reallocate(
            pointer,
            oldSize,
            layout.size,
            layout.align,
         ))
         .unwrap(),
      );
   }
}

// MODULES //

/// A simple heap based on a buddy allocator.  For the theory of buddy
/// allocators, see https://en.wikipedia.org/wiki/Buddy_memory_allocation
/// or https://www.memorymanagement.org/mmref/alloc.html#buddy-system
///
/// The basic idea is that our heap size is a power of two, and the heap
/// starts out as one giant free block.  When a memory allocation request
/// is received, we round the requested size up to a power of two, and find
/// the smallest available block we can use.  If the smallest free block is
/// too big (more than twice as big as the memory we want to allocate), we
/// split the smallest free block in half recursively until it's the right
/// size.  This simplifies a lot of bookkeeping, because all our block
/// sizes are a power of 2, which makes it easy to have one free array per
/// block size.
pub mod heap;

/// Implements a simple memory layout structure.
pub mod layout;

/// Implements two memory allocators: a Buddy Allocator and a Best-Fit Allocator,
/// respectively referred to as [`BUDDY_ALLOCATOR`][crate::alloc::paging::BUDDY_ALLOCATOR]
/// and [`FIT_ALLOCATOR`][crate::alloc::paging::FIT_ALLOCATOR].
pub mod paging;

// EXPORTS //

pub use self::layout::Layout;

// IMPORTS //

use {
   self::heap::HEAP,
   core::{
      cell::RefCell,
      cmp::min,
      fmt::{Display, Formatter},
      mem::size_of,
      ptr::{self, NonNull},
   },
   spin::{Mutex, MutexGuard},
};
