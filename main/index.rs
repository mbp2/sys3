#![allow(nonstandard_style)]
#![feature(
   abi_x86_interrupt,
   const_mut_refs,
   panic_info_message,
)]
#![no_main]
#![no_std]

/// Our bootloader configuration.
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
   let mut config = BootloaderConfig::new_default();
   config.mappings.framebuffer = Mapping::FixedAddress(0x8000000);
   config.mappings.physical_memory = Some(Mapping::Dynamic);
   config.mappings.page_table_recursive = Some(Mapping::Dynamic);
   config
};

/// System entry point.
pub fn Main(info: &'static mut BootInfo) -> ! {
   let fb_info = info.framebuffer
      .as_ref().clone().unwrap().info();

   let framebuffer = info.framebuffer.as_mut().unwrap();

   // Initialise the global descriptor table.
   gdt::initGDT();

   // Initialise the interrupt descriptor table.
   interrupts::initIDT();

   // Initialise logging facilities.
   let buffer = framebuffer.buffer_mut();
   terminal::init_writer(buffer, fb_info, true, false);

   println!("Hello world!");

   loop {}
}

/// This function is called on compiler or runtime panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
   print!("Aborting: ");
   if let Some(locale) = info.location() {
      println!(
         "@ line {}, file {}: {}",
         locale.line(),
         locale.file(),
         info.message().unwrap(),
      );
   } else {
      println!("no panic information available");
   }

   abort();
}

#[no_mangle]
extern "C" fn abort() -> ! {
   loop {
      unsafe {
         #[cfg(target_arch="aarch64")]
         core::arch::asm!();

         #[cfg(any(target_arch="riscv64", target_arch="riscv32"))]
         core::arch::asm!("wfi"::::"volatile");

         #[cfg(any(target_arch="x86", target_arch="x86_64"))]
         core::arch::asm!("hlt");
      }
   }
}

#[no_mangle]
extern "C" fn eh_personality() {}

springboard_api::entry_point!(Main, config = &BOOTLOADER_CONFIG);

// MODULES //

/// Architecture-specific code.
pub mod arch;

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

use springboard_api::config::Mapping;
use {
   base::terminal,
   core::panic::PanicInfo,
   springboard_api::{BootInfo, BootloaderConfig}
};
