/// Keeps track of used entries in a level 4 page table.
///
/// Useful for determining a free virtual memory block, e.g. for mapping additional data.
pub struct UsedLevel4Entries {
   /// Whether an entry is in use by the kernel.
   entryState: [bool; 512],
   /// A random number generator that should be used to generate random addresses or
   /// `None` if aslr is disabled.
   rng: Option<Hc128Rng>,
}

// IMPORTS //

use {
   crate::{
      api::info::{BootInfo, MemoryRegion},
      config::{self, BootConfig},
      entropy,
   },
   core::{alloc::Layout, iter::Step},
   rand::{
      distributions::{Distribution, Uniform},
      seq::IteratorRandom,
   },
   rand_hc::Hc128Rng,
   usize_conversions::IntoUsize,
   x86_64::{
      structures::paging::{Page, PageTableIndex, Size4KiB},
      PhysAddr, VirtAddr,
   },
   xmas_elf::program::ProgramHeader,
};
