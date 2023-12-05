#![allow(nonstandard_style)]
#![feature(
abi_x86_interrupt,
allocator_api,
alloc_error_handler,
async_closure,
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
   // Initialise logging facilities.
   let framebuffer = info.framebuffer.clone();
   let fb_info = framebuffer.as_ref().unwrap().info();
   let buffer = framebuffer.into_option().unwrap().into_buffer();
   terminal::init_writer(buffer, fb_info, true, false);

   // Initialise the global descriptor table.
   log::info!("Initialising global descriptor table!");
   gdt::initGDT();

   // Initialise the interrupt descriptor table.
   log::info!("Initialising interrupt descriptor table!");
   interrupts::initialise();

   // Set up our page tables.
   let physical_offset = info.physical_memory_offset.clone();
   let physical_offset = VirtAddr::new(physical_offset.into_option().unwrap());
   let mut mapper = unsafe{ memory::initialise(physical_offset) };
   let mut frame_allocator = unsafe{
      SystemFrameAllocator::new(info.memory_regions.as_ref())
   };

   log::info!("Building the heap!");
   memory::build_heap(&mut mapper, &mut frame_allocator)
      .expect("failed to initialise heap");

   // Call the runtime init process, call primary shell.
   process::initialise();

   async fn async_number() -> usize { 42 }

   let example = async || {
      let number = async_number().await;
      println!("async number: {}", number);
   };

   let mut executor = Executor::new();
   executor.spawn(Task::new(example()));
   executor.spawn(Task::new(keyboard::print_keypresses()));
   executor.run();
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

pub fn hlt_loop() -> ! {
   loop{
      x86_64::instructions::hlt();
   }
}

springboard_api::start!(Main, config = &BOOTLOADER_CONFIG);

// MODULES //

/// Advanced Programmable Interrupt Controller (APIC) implementation.
///
/// Includes CPU APIC and IO APIC implementations.
pub mod apic;

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
///
/// Includes APIC initialisation as well!
pub mod interrupts;

/// Kernel memory management.
pub mod memory;

/// Kernel-level process management.
pub mod process;

// IMPORTS //

extern crate alloc;
extern crate acpi;
#[macro_use] extern crate base;
extern crate springboard_api;
extern crate x86_64;

use {
   crate::memory::SystemFrameAllocator,
   base::{
      log,
      tasks::{
         executor::Executor,
         Task, keyboard
      },
      terminal,
   },
   core::panic::PanicInfo,
   springboard_api::{BootInfo, BootloaderConfig, config::Mapping},
   x86_64::{
      VirtAddr,
   },
};
