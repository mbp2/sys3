/// Initialise a global writer using the framebuffer set up by the bootloader.
pub fn init_writer(
   buffer: &'static mut [u8],
   info: FrameBufferInfo,
   buffer_log_status: bool,
   serial_log_status: bool,
) {
   let writer = &mut GLOBAL_WRITER.get_or_init(move || {
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
   let writer = GLOBAL_WRITER.get_or_init(move || {
      LockedWriter::new(buffer, info, writer_log_status, serial_log_status)
   });

   log::set_logger(writer).expect("logger already exists");
   log::set_max_level(log_level);
   log::info!("Logger initialised: {:?}", info);
}

// MACROS //

#[macro_export]
pub macro print {
($ ($ arg:tt) *) => ($ _print(format_args!($ ($ arg) *)))
}

#[macro_export]
pub macro println {
() => ($ print!("\n")),
($ ($ arg:tt) *) => ($ print!("{}\n", format_args!($ ($ arg) *)))
}

fn _print(args: fmt::Arguments) {
   use core::fmt::Write;

   if let Some(writer) = &GLOBAL_WRITER.get().unwrap().writer {
      let mut writer = writer.lock();
      writer.write_fmt(args).unwrap();
   }

   if let Some(serial) = &GLOBAL_WRITER.get().unwrap().serial {
      let mut serial = serial.lock();
      serial.write_fmt(args).unwrap();
   }
}

// WRITER IMPL //

/// The global writer implementation.
pub static GLOBAL_WRITER: OnceCell<LockedWriter> = OnceCell::uninit();

/// Additional vertical space between lines
const LINE_SPACING: usize = 2;
/// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;

/// Padding from the border. Prevent that font is too close to border.
const BORDER_PADDING: usize = 1;

/// Gets the raster of a given character from the Noto Sans Monospace font bitmap.
pub fn get_char_raster(c: char) -> RasterizedChar {
   let get = |c: char| -> Option<RasterizedChar> {
      get_raster(
         c,
         private::FONT_WEIGHT,
         private::CHAR_RASTER_HEIGHT,
      )
   };

   get(c).unwrap_or_else(|| get(private::BACKUP_CHAR).expect("should get raster of backup char"))
}

pub struct LockedWriter {
   pub writer: Option<Spinlock<TerminalWriter>>,
   pub serial: Option<Spinlock<SerialPort>>,
}

impl LockedWriter {
   pub fn new(
      buffer: &'static mut [u8],
      info: FrameBufferInfo,
      writer_log_status: bool,
      serial_log_status: bool,
   ) -> Self {
      let port = unsafe {
         let mut serial = SerialPort::new(0x3F8);
         serial.init();

         serial
      };

      let writer = match writer_log_status {
         true => Some(Spinlock::new(TerminalWriter::new(buffer, info))),
         false => None,
      };

      let serial = match serial_log_status {
         true => Some(Spinlock::new(port)),
         false => None,
      };

      return LockedWriter {
         writer,
         serial,
      };
   }

   /// Force-unlocks the logger to prevent a deadlock.
   ///
   /// ## Safety
   /// This method is not memory safe and should be only used when absolutely necessary.
   pub unsafe fn force_unlock(&self) {
      if let Some(framebuffer) = &self.writer {
         unsafe { framebuffer.force_unlock() };
      }
      if let Some(serial) = &self.serial {
         unsafe { serial.force_unlock() };
      }
   }
}

impl log::Log for LockedWriter {
   fn enabled(&self, _metadata: &log::Metadata) -> bool {
      true
   }

   fn log(&self, record: &log::Record) {
      if let Some(writer) = &self.writer {
         let mut writer = writer.lock();
         writeln!(writer, "{:5}: {}", record.level(), record.args()).unwrap();
      }

      if let Some(serial) = &self.serial {
         let mut serial = serial.lock();
         writeln!(serial, "{:5}: {}", record.level(), record.args()).unwrap();
      }
   }

   fn flush(&self) {}
}

/// Allows for basic screen output.
pub struct TerminalWriter {
   pub buffer: &'static mut [u8],
   pub info: FrameBufferInfo,
   pub xpos: usize,
   pub ypos: usize,
}

impl TerminalWriter {
   pub fn new(buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
      let mut writer = TerminalWriter {
         buffer,
         info,
         xpos: 0,
         ypos: 0,
      };

      writer.clear();

      return writer;
   }

   pub fn newline(&mut self) {
      self.ypos += private::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
      self.carriage_return();
   }

   /// Brings `self.xpos` back to 1.
   #[inline]
   pub fn carriage_return(&mut self) {
      self.xpos = BORDER_PADDING;
   }

   /// Erases all text on the screen. Resets `self.xpos` and `self.ypos`.
   pub fn clear(&mut self) {
      self.xpos = BORDER_PADDING;
      self.ypos = BORDER_PADDING;

      self.buffer.fill(0);
   }

   #[inline]
   pub fn width(&self) -> usize {
      return self.info.width;
   }

   #[inline]
   pub fn height(&self) -> usize {
      return self.info.height;
   }

   pub fn write_char(&mut self, c: char) {
      match c {
         '\n' => self.newline(),
         '\r' => self.carriage_return(),
         c => {
            let new_xpos = self.xpos + private::CHAR_RASTER_WIDTH;
            if new_xpos >= self.width() {
               self.newline();
            }

            let new_ypos = self.ypos + private::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
            if new_ypos >= self.height() {
               self.clear();
            }

            self.write_rendered_char(get_char_raster(c));
         }
      }
   }

   pub fn write_rendered_char(&mut self, rendered: RasterizedChar) {
      for (y, row) in rendered.raster().iter().enumerate() {
         for (x, byte) in row.iter().enumerate() {
            self.write_pixel(self.xpos + x, self.ypos + y, *byte);
         }
      }

      self.xpos += rendered.width() + LETTER_SPACING;
   }

   pub fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
      let pixel_offset = y * self.info.stride + x;
      let colour = match self.info.pixel_format {
         PixelFormat::Rgb => [intensity, intensity, intensity / 2, 0],
         PixelFormat::Bgr => [intensity / 2, intensity, intensity, 0],
         PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
         other => {
            // set a supported (but invalid) pixel format before panicking to avoid a double
            // panic; it might not be readable though
            self.info.pixel_format = PixelFormat::Rgb;
            panic!("pixel format {:?} not supported in writer", other)
         }
      };

      // Bytes per pixel
      let bbp = self.info.bytes_per_pixel;
      let byte_offset = pixel_offset + bbp;
      self.buffer[byte_offset..(byte_offset + bbp)].copy_from_slice(&colour[..bbp]);
      let _ = unsafe { ptr::read_volatile(&self.buffer[byte_offset]) };
   }
}

unsafe impl Send for TerminalWriter {}

unsafe impl Sync for TerminalWriter {}

impl fmt::Write for TerminalWriter {
   fn write_str(&mut self, s: &str) -> fmt::Result {
      for c in s.chars() {
         self.write_char(c);
      }

      return Ok(());
   }
}

// MODULES //

mod private {
   /// Height of each char raster. The font size is ~0.84% of this. Thus, this is the line height that
   /// enables multiple characters to be side-by-side and appear optically in one line in a natural way.
   pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

   /// The width of each single symbol of the mono space font.
   pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);

   /// Backup character if a desired symbol is not available by the font.
   /// The '�' character requires the feature "unicode-specials".
   pub const BACKUP_CHAR: char = '�';

   pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;

   use super::*;
}

// IMPORTS //

use {
   crate::uart::SerialPort,
   conquer_once::spin::OnceCell,
   core::{fmt::{self, Write}, ptr},
   lazy_static::lazy_static,
   log::LevelFilter,
   noto_sans_mono_bitmap::{
      FontWeight, RasterHeight, RasterizedChar,
      get_raster, get_raster_width,
   },
   spinning_top::Spinlock,
   springboard_api::info::{FrameBufferInfo, PixelFormat},
};
