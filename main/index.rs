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

// MODULES //

pub mod panic;

// IMPORTS //

extern crate base;

use {
   base::alloc::heap,
   springboard_api::{BootInfo, BootloaderConfig}
};
