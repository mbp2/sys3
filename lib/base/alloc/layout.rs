/// Defines the layout of memory to be allocated.
#[derive(Copy, Clone)]
pub struct Layout {
   #[doc(hidden)]
   pub size: usize,
   #[doc(hidden)]
   pub align: usize,
}

impl Layout {
   /// Creates a new instance of a Layout.
   #[inline]
   pub fn new<T>() -> Self {
      return Layout {
         size: size_of::<T>(),
         align: align_of::<T>(),
      };
   }

   /// Creates a new instance of a Layout with the given size.
   #[inline]
   pub fn from_size(size: usize) -> Self {
      return Layout { size, align: 4 };
   }

   /// Create a new instance of Layout from the given array-length and type parameter.
   #[inline]
   pub fn from_type_array<T>(length: usize) -> Self {
      return Layout {
         size: size_of::<T>() * length,
         align: align_of::<T>(),
      };
   }

   /// Realigns data.
   #[inline(always)]
   pub fn align_up(&self, i: usize) -> usize {
      let p = i + self.align - 1;
      return p - (p % self.align);
   }
}

#[derive(Debug)]
pub struct LayoutError;

impl Display for LayoutError {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "An error occurred for the requested layout")
   }
}

// IMPORTS //

use core::{
   fmt::{self, Display},
   mem::{align_of, size_of},
};
