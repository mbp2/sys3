#![allow(nonstandard_style)]
#![no_main]
#![no_std]

static BOOTLOADER_CONFIG: BootloaderConfig =
   {
      let config = BootloaderConfig::new_default();
      config
   };

/// System entry point.
pub fn Main(info: &'static mut BootInfo) -> ! {
   loop {}
}

springboard_api::entry_point!(Main, config = &BOOTLOADER_CONFIG);

// IMPORTS //

use springboard_api::{BootInfo, BootloaderConfig};

// MODULES //

pub mod panic;
