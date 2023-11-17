/// Additional vertical space between lines
const LINE_SPACING: usize = 2;
/// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;

/// Padding from the border. Prevent that font is too close to border.
const BORDER_PADDING: usize = 1;

/// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
fn getCharRaster(c: char) -> RasterizedChar {
   fn get(c: char) -> Option<RasterizedChar> {
      get_raster(
         c,
         font_constants::FONT_WEIGHT,
         font_constants::CHAR_RASTER_HEIGHT,
      )
   }

   get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("should get raster of backup char"))
}

/// Allows logging text to a pixel-based framebuffer.
pub struct FrameBufferWriter {
   buffer: &'static mut [u8],
   info: PixelBufferInfo,
   xPos: usize,
   yPos: usize,
}

impl FrameBufferWriter {
   pub fn new(buffer: &'static mut [u8], info: PixelBufferInfo) -> Self {
      let mut logger = FrameBufferWriter {
         buffer,
         info,
         xPos: 0,
         yPos: 0,
      };

      logger.clear();
      return logger;
   }

   fn newline(&mut self) {
      self.yPos = font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
      self.carriageReturn();
   }

   fn carriageReturn(&mut self) {
      self.xPos = BORDER_PADDING;
   }

   /// Erases all text on the screen. Resets `self.x_pos` and `self.y_pos`.
   pub fn clear(&mut self) {
      self.xPos = BORDER_PADDING;
      self.yPos = BORDER_PADDING;
      self.buffer.fill(0);
   }

   fn width(&self) -> usize {
      return self.info.width;
   }

   fn height(&self) -> usize {
      return self.info.height;
   }

   fn writeChar(&mut self, c: char) {
      match c {
         '\n' => self.newline(),
         '\r' => self.carriageReturn(),
         c => {
            let newX = self.xPos + font_constants::CHAR_RASTER_WIDTH;
            if newX >= self.width() {
               self.newline();
            }

            let newY = self.yPos + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
            if newY >= self.height() {
               self.clear();
            }

            self.writeRenderedChar(getCharRaster(c));
         }
      }
   }

   fn writeRenderedChar(&mut self, rendered: RasterizedChar) {
      for (y, row) in rendered.raster().iter().enumerate() {
         for (x, byte) in row.iter().enumerate() {
            self.writePixel(self.xPos + x, self.yPos + y, *byte);
         }
      }

      self.xPos += rendered.width() + LETTER_SPACING;
   }

   fn writePixel(&mut self, x: usize, y: usize, intensity: u8) {
      let pixelOffset = y * self.info.stride + x;
      let colour = match self.info.pixelFormat {
         PixelFormat::Rgb => [intensity, intensity, intensity / 2, 0],
         PixelFormat::Bgr => [intensity / 2, intensity, intensity, 0],
         PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
         other => {
            // set a supported (but invalid) pixel format before panicking to avoid a double
            // panic; it might not be readable though
            self.info.pixelFormat = PixelFormat::Rgb;
            panic!("pixel format {:?} not supported in logger", other)
         }
      };

      let bbp = self.info.bbp;
      let byteOffset = pixelOffset * bbp;
      self.buffer[byteOffset..(byteOffset + bbp)].copy_from_slice(&colour[..bbp]);

      let _ = unsafe { ptr::read_volatile(&self.buffer[byteOffset]) };
   }
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl fmt::Write for FrameBufferWriter {
   fn write_str(&mut self, s: &str) -> fmt::Result {
      for c in s.chars() {
         self.writeChar(c);
      }

      return Ok(());
   }
}

// IMPORTS //

use {
   crate::api::info::{PixelBufferInfo, PixelFormat},
   core::{fmt, ptr},
   font_constants::BACKUP_CHAR,
   noto_sans_mono_bitmap::{
      get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
   },
};

// MODULES //

mod font_constants {
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
