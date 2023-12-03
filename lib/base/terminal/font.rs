/// Height of each char raster. The font size is ~0.84% of this. Thus, this is the line height that
/// enables multiple characters to be side-by-side and appear optically in one line in a natural way.
pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

/// The width of each single symbol of the mono space font.
pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);

/// Backup character if a desired symbol is not available by the font.
/// The '�' character requires the feature "unicode-specials".
pub const BACKUP_CHAR: char = '�';

pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;

extern "C" {
   pub static mut _binary_font_psf_start: u64;
   pub static mut _binary_font_psf_end: u64;
}

/// The PC Screen Font header.
pub struct ScreenFontHeader {
   /// Magic bytes for font identification.
   pub magic: u16,
   /// The PSF font mode.
   pub font_mode: u8,
   /// The PSF character size.
   pub size: u8,
}

pub struct ScreenFont {
   /// Magic bytes to identify PSF.
   pub magic: u32,
   /// Zero, and nothing else.
   pub version: u32,
   /// Offset of bitmaps in file, `32`.
   pub header_size: u32,
   /// `0` if there is no unicode table.
   pub flags: u32,
   /// The number of glyphs.
   pub glyph_count: u32,
   /// THe size of each glyph.
   pub bytes_per_glyph: u32,
   /// The height in pixels of a glyph.
   pub height: u32,
   /// The width in pixels of a glyph.
   pub width: u32,
}

// IMPORTS //

use noto_sans_mono_bitmap::{
   FontWeight, RasterHeight, RasterizedChar,
   get_raster, get_raster_width,
};
