/// An allocator-aware smart pointer similar to C++'s `unique_ptr`.
///
/// # Usage
///
/// ```no_compile
/// let pointer = Unique::new(100_000);
/// let num = *pointer;
/// ```
///
/// TODO: Write better documentation.
pub struct Unique<T: ?Sized, A: Allocator = GlobalAllocator> {
   pointer: NonNull<T>,
   allocator: A,
   _ghost: PhantomData<T>,
}

impl<T, A: Allocator> Unique<T, A> {
   pub fn new_with(value: T, mut allocator: A) -> Self {
      let mut pointer =
         unsafe {
            allocate::<T>(&mut allocator)
               .expect("allocation failure")
               .cast::<T>()
         };

      unsafe {
         write::<T>(pointer.as_mut(), value);
      }

      return Unique {
         pointer,
         allocator,
         _ghost: PhantomData,
      };
   }

   pub fn pin_with(value: T, allocator: A) -> Pin<Self> {
      return unsafe { Pin::new_unchecked(Self::new_with(value, allocator)) };
   }
}

impl<T: ?Sized, A: Allocator> Unique<T, A> {
   pub unsafe fn from_raw_with(pointer: NonNull<T>, allocator: A) -> Self {
      return Unique {
         pointer,
         allocator,
         _ghost: PhantomData,
      };
   }

   pub fn leak<'a>(unique: Unique<T, A>) -> &'a mut T
   where
      A: 'a, {
      let reference = unsafe { &mut *unique.pointer.as_ptr() };
      core::mem::forget(unique);

      return reference;
   }

   pub fn into_raw(unique: Self) -> *mut T {
      let pointer = unique.pointer.as_ptr();
      core::mem::forget(unique);

      return pointer;
   }
}

impl<T> Unique<T, GlobalAllocator> {
   pub fn new(value: T) -> Self {
      return Unique::new_with(value, GlobalAllocator);
   }

   pub fn pin(value: T) -> Pin<Self> {
      return Self::pin_with(value, GlobalAllocator);
   }
}

impl<T: ?Sized, A: Allocator> Deref for Unique<T, A> {
   type Target = T;

   #[inline]
   fn deref(&self) -> &Self::Target {
      return unsafe { self.pointer.as_ref() };
   }
}

impl<T: ?Sized, A: Allocator> DerefMut for Unique<T, A> {
   fn deref_mut(&mut self) -> &mut Self::Target {
      return unsafe { self.pointer.as_mut() };
   }
}

impl<T: ?Sized, A: Allocator> AsRef<T> for Unique<T, A> {
   #[inline]
   fn as_ref(&self) -> &T {
      return self;
   }
}

impl<T: ?Sized, A: Allocator> AsMut<T> for Unique<T, A> {
   #[inline]
   fn as_mut(&mut self) -> &mut T {
      return self;
   }
}

impl<T: ?Sized, A: Allocator> Borrow<T> for Unique<T, A> {
   #[inline]
   fn borrow(&self) -> &T {
      return self;
   }
}

impl<T: ?Sized, A: Allocator> BorrowMut<T> for Unique<T, A> {
   #[inline]
   fn borrow_mut(&mut self) -> &mut T {
      return self;
   }
}

impl<T: ?Sized, A: Allocator> Drop for Unique<T, A> {
   fn drop(&mut self) {
      let size = unsafe { size_of_val(self.pointer.as_ref()) };
      let layout = Layout::from_size(size);

      unsafe {
         drop_in_place(self.pointer.as_ptr());
         self
            .allocator
            .deallocate(self.pointer.cast().as_ptr(), layout);
      }
   }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized, A: Allocator> CoerceUnsized<Unique<U, A>> for Unique<T, A> {}

impl<T: ?Sized, A: Allocator> Unpin for Unique<T, A> {}

unsafe impl<T: ?Sized + Send, A: Allocator> Send for Unique<T, A> {}
unsafe impl<T: ?Sized + Sync, A: Allocator> Sync for Unique<T, A> {}

// IMPORTS //

use {
   crate::alloc::{allocate, Allocator, GlobalAllocator, Layout},
   core::{
      borrow::{Borrow, BorrowMut},
      convert::{AsMut, AsRef},
      marker::{PhantomData, Unsize},
      mem::size_of_val,
      ops::{CoerceUnsized, Deref, DerefMut},
      pin::Pin,
      ptr::{drop_in_place, write, NonNull},
   },
};
