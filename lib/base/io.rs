/// A general I/O interface.
pub trait HardwareIo {
   /// The value type used for operations.
   type Value: Copy + PartialEq + BitAnd<Output = Self::Value> + BitOr<Output = Self::Value> + Not<Output = Self::Value>;

   /// Reads from the interface, returning the read value.
   fn read(&self) -> Self::Value;

   /// Writes value to the interface.
   fn write(&mut self, value: Self::Value);

   /// Reads the value from the I/O interface and checks if the specified flags are set.
   ///
   /// # Arguments
   ///
   /// * `flags` - The flags to check.
   ///
   /// # Returns
   ///
   /// Returns `true` if all the specified flags are set, `false` otherwise.
   #[inline(always)]
   fn read_flags(&self, flags: Self::Value) -> bool {
      return ((self.read() & flags) as Self::Value) == flags;
   }

   /// Writes the value to the I/O interface based on the specified flags and value.
   ///
   /// # Arguments
   ///
   /// * `flags` - The flags to modify.
   /// * `value` - The value indicating whether to set (`true`) or clear (`false`) the flags.
   #[inline(always)]
   fn write_flags(&mut self, flags: Self::Value, value: bool) {
      let tmp: Self::Value = match value {
         true => self.read() | flags,
         false => self.read() & !flags,
      };

      self.write(tmp);
   }
}

/// Wrapper around an I/O interface providing read-only access.
pub struct ReadOnly<I>(pub I);

impl<I> ReadOnly<I> {
   /// Creates a new `ReadOnly` wrapper instance.
   ///
   /// # Arguments
   ///
   /// * `inner` - The inner I/O interface.
   pub const fn new(inner: I) -> Self {
      return ReadOnly(inner);
   }
}

impl<I: HardwareIo> ReadOnly<I> {
   /// Reads the value from the interface.
   #[inline(always)]
   pub fn read(&self) -> I::Value {
      self.0.read()
   }

   pub fn read_flags(&self, flags: I::Value) -> bool {
      self.0.read_flags(flags)
   }
}

/// Wrapper around an I/O interface providing write-only access.
pub struct WriteOnly<I>(pub I);

impl<I> WriteOnly<I> {
   /// Writes the value to the I/O interface.
   ///
   /// # Arguments
   ///
   /// * `value` - The value to write.
   pub const fn new(inner: I) -> Self {
      return WriteOnly(inner);
   }
}

impl<I: HardwareIo> WriteOnly<I> {
   /// Writes the value to the I/O interface.
   ///
   /// # Arguments
   ///
   /// * `value` - The value to write.
   #[inline(always)]
   pub fn write(&mut self, value: I::Value) {
      self.0.write(value);
   }

   /// Writes the value to the I/O interface based on the specified flags and value.
   ///
   /// # Arguments
   ///
   /// * `flags` - The flags to modify.
   /// * `value` - The value indicating whether to set (`true`) or clear (`false`) the flags.
   #[inline(always)]
   pub fn write_flags(&mut self, flags: I::Value, value: bool) {
      self.0.write_flags(flags, value);
   }
}

// IMPORTS //

use core::{
   cmp::PartialEq,
   ops::{BitAnd, BitOr, Not},
};
