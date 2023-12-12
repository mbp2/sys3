pub fn initialise_platform() {
   log::info!("Detected RISC-V CPU!");
   log::info!("Moving to initialise RISC-V platform modules.");
}

// IMPORTS //

use base::log;

// MODULES //

pub mod pixbuf;
