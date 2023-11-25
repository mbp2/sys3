pub struct StandardWriter {
   buffer: &'static mut [u8],
   info: FrameBufferInfo,
   xpos: usize,
   ypos: usize,
}

// IMPORTS //

use {
   super::font::*,
   crate::uart::SerialPort,
   conquer_once::spin::OnceCell,
   core::{fmt::{self, Write}, ptr},
   noto_sans_mono_bitmap::{RasterizedChar, get_raster},
   spinning_top::Spinlock,
   springboard_api::info::{FrameBufferInfo, PixelFormat},
};
