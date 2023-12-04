pub fn previous_po2(number: usize) -> usize {
   return 1 << (usize::BITS as usize - number.leading_zeros() as usize - 1);
}

/// Basic power-of-2 integer math.
pub trait PowersOf2 {
   fn powerOf2(self) -> bool;
   fn nextPowerOf2(self) -> usize;
   fn log2(self) -> u8;
}

impl PowersOf2 for usize {
   /// This code is based on
   /// http://graphics.stanford.edu/~seander/bithacks.html#DetermineIfPowerOf2
   fn powerOf2(self) -> bool {
      self != 0 && (self & (self - 1)) == 0
   }

   /// Calculate the next power of two.
   ///
   /// Based on
   /// http://graphics.stanford.edu/~seander/bithacks.html#RoundUpPowerOf2
   fn nextPowerOf2(self) -> usize {
      // Pick off this immediately in hopes that the optimizer can see it
      // easily.
      if self == 0 {
         return 1;
      }

      let mut v = Wrapping(self);

      v = v - Wrapping(1);
      v = v | (v >> 1);
      v = v | (v >> 2);
      v = v | (v >> 4);
      v = v | (v >> 8);
      v = v | (v >> 16);
      if size_of::<usize>() > 4 {
         v = v | (v >> 32);
      }
      v = v + Wrapping(1);

      let result = match v {
         Wrapping(v) => v,
      };
      assert!(result.powerOf2());
      assert!(result >= self && self > result >> 1);
      result
   }

   /// Calculate the base-2 logarithm of this value.
   ///
   /// This will normally round down, except for the case of `0.log2()`,
   /// which will return 0.
   ///
   /// Based on the obvious code at
   /// http://graphics.stanford.edu/~seander/bithacks.html#IntegerLogObvious
   fn log2(self) -> u8 {
      let mut temp = self;
      let mut result = 0;
      temp >>= 1;
      while temp != 0 {
         result += 1;
         temp >>= 1;
      }
      result
   }
}

// IMPORTS //

use core::{mem::size_of, num::Wrapping};
