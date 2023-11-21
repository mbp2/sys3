/// Basic power-of-2 integer math.
pub trait PowersOf2 {
   fn PowerOf2(self) -> bool;
   fn NextPowerOf2(self) -> usize;
   fn Log2(self) -> u8;
}

impl PowersOf2 for usize {
   /// This code is based on
   /// http://graphics.stanford.edu/~seander/bithacks.html#DetermineIfPowerOf2
   fn PowerOf2(self) -> bool {
      self != 0 && (self & (self - 1)) == 0
   }

   /// Caluculate the next power of two.
   ///
   /// Based on
   /// http://graphics.stanford.edu/~seander/bithacks.html#RoundUpPowerOf2
   fn NextPowerOf2(self) -> usize {
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
      assert!(result.is_power_of_2());
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
   fn Log2(self) -> u8 {
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

use core::{
   num::Wrapping,
   mem::size_of,
};
