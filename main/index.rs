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
   let fb_info = info.framebuffer
      .as_ref().clone().unwrap().info();

   let framebuffer = info.framebuffer.as_mut().unwrap();

   /// Initialise the global descriptor table.
   gdt::initGDT();

   // Initialise the interrupt descriptor table.
   interrupts::initIDT();

   // Initialise logging facilities.
   let buffer = framebuffer.buffer_mut();
   /*TODO: this currently prints an invalid sequence, uncomment when fixed.
   terminal::init_writer(buffer, fb_info, true, false);

   println!("Hello world!");
    */

   loop {}
}

/// This function is called on compiler or runtime panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
   loop {}
}

springboard_api::entry_point!(Main, config = &BOOTLOADER_CONFIG);

// MODULES //

/// The Global Descriptor Table (GDT) is a relic that was used for memory segmentation before
/// paging became the de facto standard. However, it is still needed in 64-bit mode for various
/// things, such as kernel/user mode configuration or TSS loading.
pub mod gdt;

/// CPU exception handling.
///
/// Currently only x86(_64) interrupts are supported, however ARM and RISC-V interrupts will be
/// implemented in the future as part of platform availability expansion efforts.
pub mod interrupts;

// IMPORTS //

#[macro_use] extern crate base;
extern crate springboard_api;
extern crate x86_64;

use core::ops::Deref;
use {
   base::terminal,
   core::panic::PanicInfo,
   springboard_api::{BootInfo, BootloaderConfig}
};
