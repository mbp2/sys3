#![allow(nonstandard_style)]
#![feature(abi_x86_interrupt)]
#![no_main]
#![no_std]

#[doc(hidden)]
static BOOTLOADER_CONFIG: BootloaderConfig =
   {
      let config = BootloaderConfig::new_default();
      config
   };

/// System entry point.
pub fn Main(info: &'static mut BootInfo) -> ! {
   interrupts::initIDT();

   loop {}
}

/// This function is called on compiler or runtime panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
   loop {}
}

springboard_api::entry_point!(Main, config = &BOOTLOADER_CONFIG);

// MODULES //

/// CPU exception handling.
///
/// Currently only x86(_64) interrupts are supported, however ARM and RISC-V interrupts will be
/// implemented in the future as part of platform availability expansion efforts.
pub mod interrupts;

// IMPORTS //

#[macro_use] extern crate base;
extern crate springboard_api;
extern crate x86_64;

use {
   base::alloc::heap,
   core::panic::PanicInfo,
   springboard_api::{BootInfo, BootloaderConfig}
};
