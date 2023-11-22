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
   _ghost: PhantomData<T>
}

impl<T, A: Allocator> Unique<T, A> {
   pub fn new_with(value: T, mut allocator: A) -> Self {
      let mut pointer = unsafe{
         alloc_one::<T>(&mut allocator)
            .expect("allocation failure")
      };

      unsafe{
         write(pointer.as_mut(), value);
      }

      return Unique{
         pointer,
         allocator,
         _ghost: PhantomData,
      };
   }

   pub fn pin_with(value: T, allocator: A) -> Pin<Self> {
      return unsafe{ Pin::new_unchecked(Self::new_with(value, allocator)) };
   }
}

// IMPORTS //

use {
   crate::alloc::{
      alloc_one, Allocator, GlobalAllocator
   },
   core::{
      borrow::{Borrow, BorrowMut},
      convert::{AsMut, AsRef},
      marker::{PhantomData, Unsize},
      ops::{CoerceUnsized, Deref, DerefMut},
      pin::Pin,
      ptr::{drop_in_place, write, NonNull},
   },
};
