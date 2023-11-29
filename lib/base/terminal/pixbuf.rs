pub struct PixelBuffer {
   pub screen: &'static mut [u32],
   pub scanline: u32,
   pub width: u32,
   pub height: u32,
}

impl PixelBuffer {
   pub fn new(
      screen: *mut u32,
      scanline: u32,
      width: u32,
      height: u32
   ) -> Result<PixelBuffer, &'static str> {
      // TODO: add initialisation error handling
      let size = (scanline * height) as usize;
      let pixbuf = PixelBuffer{
         screen: unsafe{
            write_bytes(screen, 0, size);
            slice::from_raw_parts_mut(screen, size)
         },
         scanline,
         width,
         height,
      };

      return Ok(pixbuf).map_err(|_: &'static str| "error mapping pixel buffer!");
   }

   /// Puts a pixel of the specified color at the given coordinates (x, y) on the screen.
   ///
   /// # Arguments
   ///
   /// * `x` - The x-coordinate of the pixel.
   /// * `y` - The y-coordinate of the pixel.
   /// * `color` - The color value of the pixel.
   ///
   /// # Safety
   ///
   /// This function assumes that the pixel coordinates are within the screen dimensions and
   /// that the framebuffer is properly initialized.
   #[inline]
   pub fn write_pixel(&mut self, x: u32, y: u32, colour: u32) {
      // Write the color value to the framebuffer
      *unsafe { self.screen.get_unchecked_mut(((self.height - 1 - y) * self.scanline / 4 + x) as usize) } = colour;
   }

   /// Display text on the self.screen using the PC self.screen Font.
   ///
   /// # Arguments
   ///
   /// * `string` - The string to be displayed on the self.screen.
   pub fn write_out(&mut self, string: &'static str) {
   }
}

// IMPORTS //

use core::{
   ptr::{addr_of, write_bytes},
   slice
};
