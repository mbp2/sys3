/// A generic PIO interface.
#[derive(Copy, Clone)]
pub struct Pio<T> {
   port: u16,
   value: PhantomData<T>,
}

impl<T> Pio<T> {
   /// Create a new PIO instance with the specified port.
   ///
   /// # Arguments
   ///
   /// * `port` - The port number.
   ///
   /// # Returns
   ///
   /// A new `Pio` instance.
   pub const fn new(port: u16) -> Self {
      return Pio{
         port,
         value: PhantomData
      };
   }
}

/// Read/write for `byte` PIO.
impl HardwareIo for Pio<u8> {
   type Value = u8;

   fn read(&self) -> Self::Value {
      let value: u8;

      unsafe{
         #[cfg(any(target_arch="x86", target_arch="x86_64"))]
         asm!("in al, dx", in("dx") self.port, out("al") value, options(nostack, nomem, preserves_flags));

         // TODO: implement ARM and RISC-V read operations.
      }

      return value;
   }

   fn write(&mut self, value: Self::Value) {
      unsafe {
         #[cfg(any(target_arch="x86", target_arch="x86_64"))]
         asm!("out dx, al", in("dx") self.port, in("al") value, options(nostack, nomem, preserves_flags));

         // TODO: implement ARM and RISC-V write operations.
      }
   }
}

/// Read/write for `word` PIO.
impl HardwareIo for Pio<u16> {
   type Value = u16;

   fn read(&self) -> Self::Value {
      let value: u16;

      unsafe{
         #[cfg(any(target_arch="x86", target_arch="x86_64"))]
         asm!("in ax, dx", in("dx") self.port, out("ax") value, options(nostack, nomem, preserves_flags));

         // TODO: implement ARM and RISC-V word read operations.
      }

      return value;
   }

   fn write(&mut self, value: Self::Value) {
      unsafe{
         #[cfg(any(target_arch="x86", target_arch="x86_64"))]
         asm!("out dx, ax", in("dx") self.port, in("ax") value, options(nostack, nomem, preserves_flags));

         // TODO: implement ARM and RISC-V word write operations.
      }
   }
}

/// Read/write for `dword` PIO.
impl HardwareIo for Pio<u32> {
   type Value = u32;

   fn read(&self) -> Self::Value {
      let value: u32;

      unsafe{
         #[cfg(target_arch="x86_64")]
         asm!("in eax, dx", in("dx") self.port, out("eax") value, options(nostack, nomem, preserves_flags));

         // TODO: implement ARM and RISC-V dword read operations.
      }

      return value;
   }

   fn write(&mut self, value: Self::Value) {
      unsafe{
         #[cfg(target_arch="x86_64")]
         asm!("out dx, eax", in("dx") self.port, in("eax") value, options(nostack, nomem, preserves_flags));

         // TODO: implement ARM and RISC-V dword write operations.
      }
   }
}

// IMPORTS //

use {
   crate::io::HardwareIo,
   core::{
      arch::asm,
      marker::PhantomData,
   },
};
