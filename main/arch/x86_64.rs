/// Build our system peripheral setup.
pub fn initialise_platform() {
   log::info!("Detected x86_64 CPU!");
   log::info!("Moving to initialise x86_64 platform modules.");
}

// IMPORTS //

use {
   base::log,
};

// MODULES //

pub mod syscall;
