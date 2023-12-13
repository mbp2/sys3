pub static mut BOOT_INFO: Option<&'static BootInfo> = None;

pub fn get_boot_stack() -> BootStack {
   unsafe{
      let regions = BOOT_INFO.expect("get bootinfo from bootloader").memory_regions.deref();

      for index in regions {
         if index.kind == MemoryRegionKind::Usable {
            return BootStack{
               start: index.start as usize,
               end: index.end as usize,
            };
         }
      }

      panic!("Unable to determine kernel stack");
   }
}

#[derive(Clone, Copy)]
pub struct BootStack {
   start: usize,
   end: usize,
}

impl BootStack {
   pub const fn new(start: usize, end: usize) -> Self {
      return BootStack{ start, end };
   }
}

impl Stack for BootStack {
   fn top(&self) -> usize {
      return self.end - 16;
   }

   fn bottom(&self) -> usize {
      return self.start;
   }
}

// IMPORTS //

use {
   crate::memory::Stack,
   core::ops::Deref,
   springboard_api::{BootInfo, info::MemoryRegionKind},
};

// MODULES //

/// Kernel management functions.
pub mod kernel;

/// x86_64-specific memory management.
pub mod memory;
