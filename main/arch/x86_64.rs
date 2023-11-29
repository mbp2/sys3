pub static FB: Mutex<Option<PixelBuffer>> = Mutex::new(None);

/// Build our system peripheral setup.
pub fn build_peripherals() {
   // Initialise COM2 interface.
   COM2.lock().initialise();
}

// IMPORTS //

use {
   base::{
      syscall::pio::Pio,
      terminal::PixelBuffer,
      uart::{SerialPort, COM2},
   },
   spin::Mutex,
};
