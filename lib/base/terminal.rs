/// The global writer implementation.
pub static GLOBAL_WRITER: OnceCell<LockedWriter> = OnceCell::uninit();

/// The global logger.
pub static GLOBAL_LOGGER: SystemLogger = SystemLogger;

/// Initialises the global writer and optionally clears the screen.
///
/// # Arguments
///
/// * `clear_on_init` - A boolean indicating whether to clear the screen before initialising the writer.
pub fn initialise(clear_on_init: bool) {
   if clear_on_init {
      _ = LockedWriter::new().write_str("\u{001B}[2J\u{001B}[H"); // Clear screen
   }

   let _ = GLOBAL_WRITER.get_or_init(|| {
      LockedWriter::new()
   });
}

/// Initializes the logger and optionally clears the screen.
///
/// # Arguments
///
/// * `clear_on_init` - A boolean indicating whether to clear the screen before initializing the logger.
///
/// # Panics
///
/// If there is an error initializing the logger, a panic will occur with the corresponding error message.
pub fn build_logger(clear_on_init: bool) {
   if clear_on_init {
      _ = LockedWriter::new().write_str("\u{001B}[2J\u{001B}[H"); // Clear screen
   }

   let init_result = log::set_logger(&GLOBAL_LOGGER).map(|()| log::set_max_level(LevelFilter::Info));
   match init_result {
      Ok(_) => log::info!("logger initialised!"),
      Err(e) => panic!("error instantiating logger: {}", e),
   }
}

pub struct SystemLogger;

pub struct LockedWriter<'a> {
   pub serial: MutexGuard<'a, SerialPort<Pio<u8>>>
}

impl<'a> LockedWriter<'a> {
   /// Creates a new instance of the locked writer.
   ///
   /// # Example
   ///
   /// ```rust
   /// use base::terminal::LockedWriter;
   /// let writer = LockedWriter::new();
   /// ```
   pub fn new() -> Self {
      return LockedWriter{ serial: COM2.lock() };
   }

   /// Writes a byte to the serial port.
   ///
   /// # Arguments
   ///
   /// * `byte` - The byte to write.
   ///
   /// # Example
   ///
   /// ```rust
   /// use base::terminal::LockedWriter;
   /// let mut writer = LockedWriter::new();
   /// writer.write_byte(b'A');
   /// ```
   pub fn write_byte(&mut self, byte: u8) {
      self.serial.write(byte);
   }
}

impl<'a> Write for LockedWriter<'a> {
   fn write_str(&mut self, s: &str) -> fmt::Result {
      for byte in s.bytes() {
         self.write_byte(byte);
      }

      return Ok(());
   }
}

impl log::Log for SystemLogger {
   fn enabled(&self, metadata: &log::Metadata) -> bool {
      // Check if given log level is enabled
      return metadata.level() <= Level::Info;
   }

   fn log(&self, record: &log::Record) {
      if self.enabled(record.metadata()) {
         println!("[{}] {}", record.level(), record.args());
      }
   }

   fn flush(&self) {}
}

// MACROS //

#[macro_export]
macro_rules! print {
   ($($args:tt)+) => ({
      use core::fmt::Write;
      let _ = $crate::terminal::LockedWriter::new().write_fmt(format_args!($($args)*)).expect("fmt print failed");
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

// MODULES //

/// Font-related constants.

pub mod font;

/// A framebuffer-based writer implementation.
pub mod pixbuf;

/// A writer implementation that piggy-backs off the framebuffer set up by the bootloader.
pub mod standard;

// IMPORTS //

use {
   crate::{
      syscall::pio::Pio,
      uart::{COM2, SerialPort},
      println, print
   },
   conquer_once::spin::OnceCell,
   core::fmt::{self, Write},
   log::{Level, LevelFilter},
   spin::MutexGuard,
};

// EXPORTS //

pub use self::{
   pixbuf::PixelBuffer,
   standard::TerminalWriter,
};
