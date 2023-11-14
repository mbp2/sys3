//! # Springboard
//! — A bootloader for the Trident system.
#![allow(non_snake_case)]
#![warn(missing_docs)]
#![deny(missing_abi)]
#![feature(step_trait)]
#![no_std]

pub const PAGE_SIZE: u64 = 4096;

pub fn LoadAndSwitchToKernel<I, D>(
   kernel: Kernel,
   bootConfig: BootConfig,
   mut frameAllocator: LegacyFrameAllocator<I, D>,
   mut pageTables: PageTables,
   systemInfo: SystemInfo,
) -> !
where
   I: ExactSizeIterator<Item = D> + Clone,
   D: LegacyMemoryRegion, {
   let config = &kernel.config;
   let mut mappings = SetUpMappings(
      kernel,
      &mut frameAllocator,
      &mut pageTables,
      systemInfo.framebuffer.as_ref(),
      config,
      &systemInfo,
   );

   let bootInfo = CreateBootInfo(
      &config,
      &bootConfig,
      frameAllocator,
      &mut pageTables,
      &mut mappings,
      systemInfo,
   );

   SwitchToKernel(pageTables, mappings, bootInfo);
}

pub fn SetUpMappings<I, D>(
   kernel: Kernel,
   frameAllocator: &mut LegacyFrameAllocator<I, D>,
   pageTables: &mut PageTables,
   framebuffer: Option<&RawPixelBufferInfo>,
   config: &BootConfig,
   systemInfo: &SystemInfo,
) -> Mappings
   where
      I: ExactSizeIterator<Item = D> + Clone,
      D: LegacyMemoryRegion,
{
   let kernelPageTable = &mut pageTables.kernel;

   let mut usedEntries = UsedLevel4Entries::new(
      frameAllocator.maxPhysAddress(),
      frameAllocator.length(),
      framebuffer,
      config,
   );

   // Enable support for the no-execute bit in page tables.
   enableNxeBit();
   // Make the kernel respect the write-protection bits even when in ring 0 by default
   enableWriteProtectBit();

   let config = &kernel.config;
   let kernelSliceStart = PhysAddr::new(kernel.startAddress as _);
   let kernelSliceLength = u64::try_from(kernel.length).unwrap();
   //TODO: Finish
}

pub fn CreateBootInfo<I, D>(
   config: &LoaderConfig,
   bootConfig: &BootConfig,
   mut frameAllocator: LegacyFrameAllocator<I, D>,
   pageTables: &mut PageTables,
   mappings: &mut Mappings,
   systemInfo: SystemInfo,
) -> &'static mut BootInfo
   where
      I: ExactSizeIterator<Item = D> + Clone,
      D: LegacyMemoryRegion,
{
   log::info!("Allocate boot info!");
}

/// The memory addresses required for the context shift.
pub struct Addresses {
   pageTable: PhysFrame,
   stackTop: VirtAddr,
   entryPoint: VirtAddr,
   bootInfo: &'static mut BootInfo,
}

fn mappingAddressPageAligned(
   mapping: Mapping,
   size: u64,
   usedEntries: &mut UsedLevel4Entries,
   kind: &str,
) -> Page {
   match mappingAddress(mapping, size, Size4KiB::SIZE, usedEntries) {
      Ok(address) => Page::from_start_address(address).unwrap(),
      Err(address) => panic!("{kind} address must be page-aligned (is `{address:?}`)"),
   }
}

fn mappingAddress(
   mapping: Mapping,
   size: u64,
   alignment: u64,
   usedEntries: &mut UsedLevel4Entries
) -> Result<VirtAddr, VirtAddr> {
   let address = match mapping {
      Mapping::Fixed(address) => VirtAddr::new(address),
      Mapping::Dynamic => usedEntries.getFreeAddress(size, alignment),
   };

   return if address.is_aligned(alignment) {
      Ok(address)
   } else {
      Err(address)
   };
}

fn enableNxeBit() {
   use x86_64::registers::control::{Efer, EferFlags};
   unsafe { Efer::update(|efer| *efer |= EferFlags::NO_EXECUTE_ENABLE) }
}

fn enableWriteProtectBit() {
   use x86_64::registers::control::{Cr0, Cr0Flags};
   unsafe { Cr0::update(|cr0| *cr0 |= Cr0Flags::WRITE_PROTECT) }
}

/// Required system information that should be queried from the BIOS or UEFI firmware.
#[derive(Debug, Copy, Clone)]
pub struct SystemInfo {
   /// Information about the (still unmapped) framebuffer.
   pub framebuffer: Option<RawPixelBufferInfo>,
   /// Address of the _Root System Description Pointer_ structure of the ACPI standard.
   pub rsdpAddress: Option<PhysAddr>,
   pub ramdiskAddress: Option<u64>,
   pub ramdiskLength: u64,
}

/// The physical address of the framebuffer and information about the framebuffer.
#[derive(Debug, Copy, Clone)]
pub struct RawPixelBufferInfo {
   /// Start address of the pixel-based framebuffer.
   pub address: PhysAddr,

   /// Information about the framebuffer, including layout and pixel format.
   pub info: PixelBufferInfo,
}

pub struct Kernel<'a> {
   pub elf: ElfFile<'a>,
   pub config: BootConfig,
   pub startAddress: *const u8,
   pub length: usize,
}

impl<'a> Kernel<'a> {
   pub fn parse(kernelSlice: &'a [u8]) -> Self {
      let kernelElf = ElfFile::new(kernelSlice).unwrap();

      let config = {
         let section = kernelElf.find_section_by_name(".bootloader-config").expect(
            "bootloader config section not found; kernel must be compiled against springboard",
         );

         let raw = section.raw_data(&kernelElf);

         LoaderConfig::Deserialise(raw)
            .expect("kernel was compiled with incompatible springboard version")
      };

      return Kernel{
         elf: kernelElf,
         config,
         startAddress: kernelSlice.as_ptr(),
         length: kernelSlice.len(),
      };
   }
}

/// Provides access to the page tables of the bootloader and kernel address space.
pub struct PageTables {
   /// Provides access to the page tables of the bootloader address space.
   pub bootloader: OffsetPageTable<'static>,

   /// Provides access to the page tables of the kernel address space (not active).
   pub kernel: OffsetPageTable<'static>,

   /// The physical frame where the level 4 page table of the kernel address space is stored.
   ///
   /// Must be the page table that the `kernel` field of this struct refers to.
   ///
   /// This frame is loaded into the `CR3` register on the final context switch to the kernel.
   pub kernelLevel4Frame: PhysFrame,
}

// IMPORTS //

use x86_64::registers::control::{Cr0, Efer};
use {
   crate::{
      api::info::{BootInfo, PixelBuffer, PixelBufferInfo},
      config::{BootConfig, LoaderConfig, Mappings, Mapping},
      legacy::{LegacyFrameAllocator, LegacyMemoryRegion},
      level4::UsedLevel4Entries,
   },
   core::{alloc::Layout, arch::asm, mem::MaybeUninit, slice},
   usize_conversions::FromUsize,
   x86_64::{
      structures::paging::{
         page_table::PageTableLevel, FrameAllocator, Mapper, OffsetPageTable, Page, PageSize,
         PageTableFlags, PageTableIndex, PhysFrame, Size2MiB, Size4KiB,
      },
      PhysAddr, VirtAddr,
   },
   xmas_elf::ElfFile,
};

// MODULES //

pub mod api;
pub mod config;
pub mod entropy;
pub mod framebuffer;
pub mod gdt;
pub mod legacy;
pub mod level4;
pub mod loader;

pub(crate) mod concat {
   include!(concat!(env!("OUT_DIR"), "/concat.rs"));
}

pub(crate) mod version_info {
   include!(concat!(env!("OUT_DIR"), "/version_info.rs"));
}

extern crate base;
extern crate conquer_once;
extern crate log;
extern crate noto_sans_mono_bitmap;
extern crate rand;
extern crate rand_hc;
extern crate serde;
extern crate spinning_top;
extern crate uart_16550;
extern crate usize_conversions;
