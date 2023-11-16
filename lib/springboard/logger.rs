pub static LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();

pub struct LockedLogger {
   pub framebuffer: Option<Spinlock<FrameBufferWriter>>,
   pub serial: Option<Spinlock<PortWrapper>>,
}

impl LockedLogger {
   pub fn new(
      framebuffer: &'static mut [u8],
      info: PixelBufferInfo,
      fbLoggerStatus: bool,
      serialLoggerStatus: bool,
   ) -> Self {
      let framebuffer = match fbLoggerStatus {
         true => Some(Spinlock::new(FrameBufferWriter::new(framebuffer, info))),
         false => None,
      };

      let serial = match serialLoggerStatus {
         true => Some(Spinlock::new(unsafe { PortWrapper::initialise() })),
         false => None,
      };

      return LockedLogger {
         framebuffer,
         serial,
      };
   }

   /// Forcefully unlocks the logger to prevent a deadlock.
   ///
   /// # Safety
   /// This method is ***NOT*** memory safe and should be used only when absolutely necessary.
   pub unsafe fn ForceUnlock(&self) {
      if let Some(framebuffer) = &self.framebuffer {
         unsafe { framebuffer.force_unlock() };
      }

      if let Some(serial) = &self.serial {
         unsafe { serial.force_unlock() };
      }
   }
}

impl log::Log for LockedLogger {
   fn enabled(&self, _: &Metadata) -> bool {
      true
   }

   fn log(&self, record: &Record) {
      if let Some(framebuffer) = &self.framebuffer {
         let mut buffer = framebuffer.lock();
         writeln!(buffer, "{:5}: {}", record.level(), record.args()).unwrap();
      }

      if let Some(serial) = &self.serial {
         let mut serial = serial.lock();
         writeln!(serial, "{:5}: {}", record.level(), record.args()).unwrap();
      }
   }

   fn flush(&self) {}
}

// IMPORTS //

use log::{Metadata, Record};
use {
   crate::{api::PixelBufferInfo, framebuffer::FrameBufferWriter, serial::PortWrapper},
   conquer_once::spin::OnceCell,
   core::fmt::Write,
   spinning_top::Spinlock,
};
