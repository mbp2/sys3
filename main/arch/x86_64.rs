/// Build our system peripheral setup.
pub fn initialise_platform() {
   log::info!("Detected x86_64 CPU!");
   log::info!("Moving to initialise x86_64 platform modules.");

   COM2.lock().initialise();
}

// IMPORTS //

use {
   base::{
      log,
      uart::COM2,
   },
};

// MODULES //

pub mod syscall;
