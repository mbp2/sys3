#![allow(nonstandard_style)]
#![feature(
abi_x86_interrupt,
allocator_api,
alloc_error_handler,
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

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

/// System entry point.
pub fn Main(info: &'static mut BootInfo) -> ! {
   // Initialise the global descriptor table.
   gdt::initGDT();

   // Initialise the interrupt descriptor table.
   interrupts::initIDT();

   // Initialise logging facilities.
   let framebuffer = info.framebuffer.clone();
   let fb_info = framebuffer.as_ref().unwrap().info();
   let buffer = framebuffer.into_option().unwrap().into_buffer();
   terminal::init_writer(buffer, fb_info, true, false);

   println!(
      r#"
      Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Et malesuada fames ac turpis. Dictum sit amet justo donec enim diam vulputate ut pharetra. Habitant morbi tristique senectus et netus et malesuada fames. Magna fermentum iaculis eu non diam phasellus. Fermentum odio eu feugiat pretium. Aenean et tortor at risus viverra adipiscing at. Id cursus metus aliquam eleifend mi in nulla posuere sollicitudin. Ut ornare lectus sit amet est placerat in egestas. Lectus vestibulum mattis ullamcorper velit sed ullamcorper morbi tincidunt ornare. Sed vulputate odio ut enim. Sem integer vitae justo eget magna fermentum iaculis. Rutrum quisque non tellus orci ac auctor augue.

      Feugiat vivamus at augue eget arcu dictum varius duis at. Pulvinar sapien et ligula ullamcorper malesuada proin libero. Consectetur libero id faucibus nisl tincidunt eget. Libero id faucibus nisl tincidunt eget nullam. Suspendisse sed nisi lacus sed viverra tellus in. Habitant morbi tristique senectus et netus et malesuada. Faucibus turpis in eu mi bibendum neque egestas congue. Purus in massa tempor nec feugiat nisl pretium fusce. Sit amet luctus venenatis lectus magna fringilla. Ac orci phasellus egestas tellus. Eu augue ut lectus arcu bibendum at varius vel. Amet luctus venenatis lectus magna. Quis vel eros donec ac odio tempor orci dapibus ultrices. Dignissim enim sit amet venenatis urna cursus. Auctor elit sed vulputate mi sit amet mauris.

      Tincidunt arcu non sodales neque. Etiam tempor orci eu lobortis elementum nibh tellus molestie nunc. At elementum eu facilisis sed odio. Venenatis lectus magna fringilla urna porttitor rhoncus dolor purus non. Auctor urna nunc id cursus metus aliquam. Iaculis at erat pellentesque adipiscing commodo elit. Ultrices gravida dictum fusce ut placerat orci. Bibendum neque egestas congue quisque egestas diam in arcu. Suspendisse in est ante in nibh mauris cursus mattis. Facilisis magna etiam tempor orci eu lobortis elementum. Tempus iaculis urna id volutpat lacus laoreet. Justo nec ultrices dui sapien eget mi proin. Elit scelerisque mauris pellentesque pulvinar pellentesque habitant morbi. Vitae elementum curabitur vitae nunc sed.

      Dis parturient montes nascetur ridiculus mus. Rutrum quisque non tellus orci ac auctor augue. Congue quisque egestas diam in arcu cursus euismod. Leo in vitae turpis massa. Vulputate mi sit amet mauris commodo quis imperdiet massa. In hac habitasse platea dictumst quisque sagittis purus sit. Ut ornare lectus sit amet est placerat. Iaculis urna id volutpat lacus laoreet. Ac turpis egestas sed tempus urna et pharetra pharetra massa. Nibh tellus molestie nunc non blandit massa.

      Sagittis orci a scelerisque purus semper. Nulla pellentesque dignissim enim sit. Elementum curabitur vitae nunc sed velit dignissim sodales ut eu. Sagittis id consectetur purus ut faucibus pulvinar elementum integer enim. Mi ipsum faucibus vitae aliquet nec. Ac tincidunt vitae semper quis lectus nulla at volutpat diam. Lectus vestibulum mattis ullamcorper velit sed. Nisi est sit amet facilisis magna etiam tempor orci. In cursus turpis massa tincidunt dui. Auctor neque vitae tempus quam pellentesque nec nam. Duis at consectetur lorem donec massa sapien.
      "#
   );

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
         #[cfg(target_arch = "aarch64")]
         core::arch::asm!("wfi"::::"volatile");

         #[cfg(target_arch = "riscv64")]
         core::arch::asm!("wfi"::::"volatile");

         #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
         core::arch::asm!("hlt");
      }
   }
}

#[no_mangle]
extern "C" fn eh_personality() {}

springboard_api::start!(Main, config = &BOOTLOADER_CONFIG);

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

/// Kernel memory management.
pub mod memory;

// IMPORTS //

//extern crate alloc;
#[macro_use] extern crate base;
extern crate springboard_api;
extern crate x86_64;

use {
   base::{
      alloc::heap,
      terminal,
   },
   core::panic::PanicInfo,
   springboard_api::{BootInfo, BootloaderConfig, config::Mapping},
};
