#![allow(nonstandard_style)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(async_closure)]
#![feature(cfg_match)]
#![feature(const_mut_refs)]
#![feature(custom_test_frameworks)]
#![feature(panic_info_message)]
#![reexport_test_harness_main="test_main"]
#![test_runner(base::test::test_runner)]
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
pub fn main(info: &'static mut BootInfo) -> ! {
   // Initialise logging facilities.
   let framebuffer = info.framebuffer.clone();
   let fb_info = framebuffer.as_ref().unwrap().info();
   let buffer = framebuffer.into_option().unwrap().into_buffer();
   terminal::init_writer(buffer, fb_info, true, false);

   // Initialise the global descriptor table.
   log::info!("Initialising global descriptor table!");
   gdt::initialise();

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
   memory::build_heap(&mut mapper, &mut frame_allocator).expect("failed to initialise heap");

   // Check CPU architecture and perform the proper initialisation.
   log::info!("Checking CPU architecture...");
   
   #[cfg(target_arch = "x86_64")]
   arch::x86_64::initialise_platform();

   #[cfg(target_arch = "riscv64")]
   arch::riscv::initialise_platform();

   #[cfg(target_arch = "aarch64")]
   arch::aarch64::initialise_platform();

   // Example multitasking
   log::info!("Checking runtime multitasking...");

   async fn example(number: u32) -> u32 {
      number + 1
   }

   tasks::add_future(async{
      let number = example(4).await;
      print!("{}", number);
   });

   //tasks::add_future(tasks::keyboard::print_keypresses());
   //Keyboard input still not working properly :(

   tasks::run_tasks(); // works now! :D

   // Our test harness.
   // Only called when running tests.
   #[cfg(test)]
   test_main();

   hlt_loop();
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

springboard_api::start!(main, config = &BOOTLOADER_CONFIG);

// MODULES //

/// Important memory addresses, address-space utilities.
pub mod address;

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
/// Optionally this module includes APIC initialisation.
pub mod interrupts;

/// Kernel memory management.
pub mod memory;

/// Kernel-level process management.
pub mod process;

#[cfg(test)]
pub mod tests;

// IMPORTS //

extern crate alloc;
extern crate acpi;
#[macro_use] extern crate base;
extern crate springboard_api;
extern crate x86_64;


use {
   crate::memory::SystemFrameAllocator,
   base::{log, tasks, terminal},
   core::panic::PanicInfo,
   springboard_api::{BootInfo, BootloaderConfig, config::Mapping},
   x86_64::VirtAddr,
};
