pub fn init_logger(buffer: &'static mut [u8], info: FrameBufferInfo) -> SystemLogger {
   return SystemLogger{
      buffer,
      info,
   };
}

pub struct SystemLogger {
   buffer: &'static mut [u8],
   info: FrameBufferInfo,
}

// IMPORTS //

use springboard_api::info::FrameBufferInfo;
