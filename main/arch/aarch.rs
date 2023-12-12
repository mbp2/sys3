pub fn initialise_platform() {
   log::info!("Detected 64-bit ARM CPU!");
   log::info!("Moving to initialise platform modules.");
}

// IMPORTS //

use base::log;

// MODULES //

pub mod pixbuf;
