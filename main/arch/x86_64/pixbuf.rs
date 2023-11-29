pub struct PixelBuffer {
   pub screen: &'static mut [u32],
   pub scanline: usize,
   pub width: usize,
   pub height: usize,
}

impl PixelBuffer {
   pub fn new(
      screen: *mut u32,
      scanline: usize,
      width: usize,
      height: usize
   ) -> Result<PixelBuffer, &'static str> {
      // TODO: add initialisation error handling
      let size = scanline * height;
      let pixbuf = PixelBuffer{
         screen: unsafe{
            write_bytes(screen, 0, size);
            slice::from_raw_parts_mut(screen, size)
         },
         scanline,
         width,
         height,
      };

      return Ok(pixbuf).map_err(|_| "error mapping pixel buffer!");
   }
}

// IMPORTS //

use core::{
   ptr::{addr_of, write_bytes},
   slice
};
