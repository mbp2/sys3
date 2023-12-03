/// Initialise a global writer using the framebuffer set up by the bootloader.
pub fn init_writer(
   buffer: &'static mut [u8],
   info: FrameBufferInfo,
   buffer_log_status: bool,
   serial_log_status: bool,
) {
   let writer = FB_WRITER.get_or_init(move || {
      LockedWriter::new(buffer, info, buffer_log_status, serial_log_status)
   });

   writer.writer.as_ref().unwrap()
      .lock()
      .write_str("Writer initialised!")
      .unwrap();
}

/// Initialise a text-based logger using the framebuffer set up by the bootloader.
pub fn init_logger(
   buffer: &'static mut [u8],
   info: FrameBufferInfo,
   log_level: LevelFilter,
   writer_log_status: bool,
   serial_log_status: bool,
) {
   let writer = FB_WRITER.get_or_init(move || {
      LockedWriter::new(buffer, info, writer_log_status, serial_log_status)
   });

   log::set_logger(writer).expect("logger already exists");
   log::set_max_level(log_level);
   log::info!("Logger initialised: {:?}", info);
}

// MACROS //

#[macro_export]
macro_rules! print {
   ($($args:tt)+) => ({
      use core::fmt::Write;

      if let Some(writer) = &crate::terminal::framebuffer::GLOBAL_WRITER.get().unwrap().writer {
         let mut writer = writer.lock();
         let _ = write!(writer, $($args)+).unwrap();
      }

      if let Some(serial) = &crate::terminal::framebuffer::GLOBAL_WRITER.get().unwrap().serial {
         let mut serial = serial.lock();
         let _ = write!(serial, $($args)+).unwrap();
      }
   });
}

#[macro_export]
macro_rules! println {
   () => ({
      print!("\n");
   });

   ($fmt:expr) => ({
      print!(concat!($fmt, "\r\n"))
   });

   ($fmt:expr, $($args:tt)+) => ({
      print!(concat!($fmt, "\r\n"), $($args)+)
   });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
   use core::fmt::Write;

   if let Some(writer) = &FB_WRITER.get().unwrap().writer {
      let mut writer = writer.lock();
      writer.write_fmt(args).unwrap();
   }

   if let Some(serial) = &FB_WRITER.get().unwrap().serial {
      let mut serial = serial.lock();
      serial.write_fmt(args).unwrap();
   }
}

// MODULES //

/// Font-related constants.
pub mod font;

/// A framebuffer-based writer implementation.
pub mod framebuffer;

/// A writer implementation that piggy-backs off the framebuffer set up by the bootloader.
pub mod standard;

// IMPORTS //

use {
   self::framebuffer::{GLOBAL_WRITER as FB_WRITER, LockedWriter},
   core::fmt::{self, Write},
   log::LevelFilter,
   springboard_api::info::{FrameBufferInfo, PixelFormat},
};
