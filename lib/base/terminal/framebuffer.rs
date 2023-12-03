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
         FONT_WEIGHT,
         CHAR_RASTER_HEIGHT,
      )
   };

   get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("should get raster of backup char"))
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
/// Allows logging text to a pixel-based framebuffer.
pub struct TerminalWriter {
   buffer: &'static mut [u8],
   info: FrameBufferInfo,
   xpos: usize,
   ypos: usize,
}

impl TerminalWriter {
   /// Creates a new logger that uses the given framebuffer.
   pub fn new(buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
      let mut logger = Self {
         buffer,
         info,
         xpos: 0,
         ypos: 0,
      };
      logger.clear();
      logger
   }

   fn newline(&mut self) {
      self.ypos += CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
      self.carriage_return()
   }

   fn carriage_return(&mut self) {
      self.xpos = BORDER_PADDING;
   }

   /// Erases all text on the screen. Resets `self.x_pos` and `self.y_pos`.
   pub fn clear(&mut self) {
      self.xpos = BORDER_PADDING;
      self.ypos = BORDER_PADDING;
      self.buffer.fill(0);
   }

   pub fn width(&self) -> usize {
      self.info.width
   }

   pub fn height(&self) -> usize {
      self.info.height
   }

   /// Writes a single char to the framebuffer. Takes care of special control characters, such as
   /// newlines and carriage returns.
   fn write_char(&mut self, c: char) {
      match c {
         '\n' => self.newline(),
         '\r' => self.carriage_return(),
         c => {
            let new_xpos = self.xpos + CHAR_RASTER_WIDTH;
            if new_xpos >= self.width() {
               self.newline();
            }
            let new_ypos = self.ypos + CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;

            if new_ypos >= self.height() {
               self.clear();
            }

            self.write_rendered_char(get_char_raster(c));
         }
      }
   }

   /// Prints a rendered char into the framebuffer.
   /// Updates `self.x_pos`.
   fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
      for (y, row) in rendered_char.raster().iter().enumerate() {
         for (x, byte) in row.iter().enumerate() {
            self.write_pixel(self.xpos + x, self.ypos + y, *byte);
         }
      }
      self.xpos += rendered_char.width() + LETTER_SPACING;
   }

   fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
      let pixel_offset = y * self.info.stride + x;
      let color = match self.info.pixel_format {
         PixelFormat::Rgb => [intensity, intensity, intensity / 2, 0],
         PixelFormat::Bgr => [intensity / 2, intensity, intensity, 0],
         PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
         other => {
            // set a supported (but invalid) pixel format before panicking to avoid a double
            // panic; it might not be readable though
            self.info.pixel_format = PixelFormat::Rgb;
            panic!("pixel format {:?} not supported in logger", other)
         }
      };
      let bytes_per_pixel = self.info.bytes_per_pixel;
      let byte_offset = pixel_offset * bytes_per_pixel;
      self.buffer[byte_offset..(byte_offset + bytes_per_pixel)]
         .copy_from_slice(&color[..bytes_per_pixel]);
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
      Ok(())
   }
}
// IMPORTS //

use {
   super::font::{
      BACKUP_CHAR,
      CHAR_RASTER_HEIGHT,
      CHAR_RASTER_WIDTH,
      FONT_WEIGHT,
   },
   crate::uart::SerialPort,
   conquer_once::spin::OnceCell,
   core::{fmt::{self, Write}, ptr},
   noto_sans_mono_bitmap::{RasterizedChar, get_raster},
   spinning_top::Spinlock,
   springboard_api::info::{FrameBufferInfo, PixelFormat},
};
