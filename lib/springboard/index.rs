//! # Springboard
//! — A bootloader for the Trident system.
#![allow(non_snake_case)]
#![warn(missing_docs)]
#![deny(missing_abi)]
#![feature(decl_macro, step_trait)]
#![no_std]

/// Defines the entry point function.
///
/// The function must have the signature `fn(&'static mut BootInfo) -> !`.
///
/// This macro just creates a function named `_start`, which the linker will use as the entry
/// point. The advantage of using this macro instead of providing your own `_start` function is
/// that the macro ensures that the function and argument types are correct.
///
/// ## Configuration
///
/// This macro supports an optional second parameter to configure how the bootloader should
/// boot the kernel. The second parameter needs to be given as `config = ...` and be of type
/// [`&BootloaderConfig`](BootloaderConfig). If not given, the configuration defaults to
/// [`BootloaderConfig::new`](BootloaderConfig::new).
///
/// ## Examples
///
/// - With default configuration:
///
///   ```no_run
///   #![no_std]
///   #![no_main]
///   # #![feature(lang_items)]
///
///   springboard::start!(main);
///
///   fn main(bootinfo: &'static mut springboard::BootInfo) -> ! {
///       loop {}
///   }
///
///   #[panic_handler]
///   fn panic(_info: &core::panic::PanicInfo) -> ! {
///       loop {}
///   }
///
///   # #[lang = "eh_personality"] fn eh_personality() {} // not needed when disabling unwinding
///   ```
///
///   The name of the entry point function does not matter. For example, instead of `main`, we
///   could also name it `fn my_entry_point(...) -> !`. We would then need to specify
///   `start!(my_entry_point)` of course.
///
/// - With custom configuration:
///
///   ```no_run
///   #![no_std]
///   #![no_main]
///   # #![feature(lang_items)]
///
///   use springboard::{start, BootloaderConfig, BootInfo};
///
///   pub static BOOTLOADER_CONFIG: BootloaderConfig = {
///       let mut config = BootloaderConfig::new_default();
///       config.kernel_stack_size = 90 * 1024;
///       config
///   };
///
///   start!(main, config = &BOOTLOADER_CONFIG);
///
///   fn main(bootinfo: &'static mut BootInfo) -> ! {
///       loop {}
///   }
///
///   #[panic_handler]
///   fn panic(_info: &core::panic::PanicInfo) -> ! {
///       loop {}
///   }
///
///   # #[lang = "eh_personality"] fn eh_personality() {} // not needed when disabling unwinding
///   ```
///
/// ## Implementation Notes
///
/// - **Start function:** The `entry_point` macro generates a small wrapper function named
///   `_start` (without name mangling) that becomes the actual entry point function of the
///   executable. This function doesn't do anything itself, it just calls into the function
///   that is provided as macro argument. The purpose of this function is to use the correct
///   ABI and parameter types required by this crate. A user-provided `_start` function could
///   silently become incompatible on dependency updates since the Rust compiler cannot
///   check the signature of custom entry point functions.
/// - **Configuration:** Behind the scenes, the configuration struct is serialized using
///   [`BootloaderConfig::serialize`](crate::BootloaderConfig::serialize). The resulting byte
///   array is then stored as a static variable annotated with
///   `#[link_section = ".loader-config"]`, which instructs the Rust compiler to store it
///   in a special section of the resulting ELF executable. From there, the bootloader will
///   automatically read it when loading the kernel.
pub macro start {
   ($path:path) => {
      $crate::start!($path, config = &$crate:config::LoaderConfig::new());
   }

   ($path:path, config = $config:expr) => {
      const _: () = {
         #[link_section=".loader-config"]
         pub static __BOOTLOADER_CONFIG: [u8; $crate::BootloaderConfig::SERIALIZED_LEN] = {
            // Validate the config
            let config &$crate::BootloaderConfig = $config;
            config.Serialise();
         };

         #[export_name="_start"]
         pub extern "C" fn __impl_start(bootInfo: &'static mut $crate::BootInfo) -> ! {
            let f: fn(&'static mut $crate::BootInfo) -> ! = $path;
            $crate::__forceuse(&__BOOTLOADER_CONFIG);
            f(bootInfo)
         }
      };
   }
}

#[doc(hidden)]
fn __forceuse(slice: &[u8]) {
   let force = slice.as_ptr() as usize;
   unsafe { asm!("add {0}, 0", in(reg) force, options(nomem, nostack)) };
}

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
      config,
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
   config: &LoaderConfig,
   systemInfo: &SystemInfo,
) -> KernelMappings
where
   I: ExactSizeIterator<Item = D> + Clone,
   D: LegacyMemoryRegion, {
   let kernelPageTable = &mut pageTables.kernel;

   let mut usedEntries = UsedLevel4Entries::new(
      frameAllocator.MaxPhysAddress(),
      frameAllocator.Length(),
      framebuffer,
      config,
   );

   // Enable support for the no-execute bit in page tables.
   enableNxeBit();
   // Make the kernel respect the write-protection bits even when in ring 0 by default
   enableWriteProtectBit();

   let config = kernel.config;
   let kernelSliceStart = PhysAddr::new(kernel.startAddress as _);
   let kernelSliceLength = u64::try_from(kernel.length).unwrap();

   let (kernelImageOffset, entryPoint, tlsTemplate) =
      loader::LoadKernel(kernel, kernelPageTable, frameAllocator, &mut usedEntries)
         .expect("no entry point");

   log::info!("Entry point at {:#x}", entryPoint.as_u64());

   // Create our stack.
   let stackStart = {
      // Page-alignment is necessary because we need a guard page directly below the stack.
      let guardPage = mappingAddressPageAligned(
         config.mappings.kernelStack,
         Size4KiB::SIZE + config.kernelStackSize,
         &mut usedEntries,
         "kernel stack start",
      );

      guardPage + 1
   };

   let stackEndAddress = stackStart.start_address() + config.kernelStackSize;
   let stackEnd = Page::containing_address(stackEndAddress - 1u64);
   for page in Page::range_inclusive(stackStart, stackEnd) {
      let frame = frameAllocator
         .allocate_frame()
         .expect("frame allocation failed when mapping kernel stack");

      let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

      match unsafe { kernelPageTable.map_to(page, frame, flags, frameAllocator) } {
         Ok(tlb) => tlb.flush(),
         Err(err) => panic!("failed to map page {:?}: {:?}", page, err),
      }
   }

   let ctxSwitchFn = PhysAddr::new(contextSwitch as *const () as u64);
   let ctxSwitchFnStartFrame = PhysFrame::containing_address(ctxSwitchFn);

   for frame in PhysFrame::range_inclusive(ctxSwitchFnStartFrame, ctxSwitchFnStartFrame + 1) {
      match unsafe { kernelPageTable.identity_map(frame, PageTableFlags::PRESENT, frameAllocator) }
      {
         Ok(tlb) => tlb.flush(),
         Err(err) => panic!("failed to identity map frame {:?}: {:?}", frame, err),
      }
   }

   // Create, load, and identity-map GDT frame.
   // This is required for working `iretq`
   let gdtFrame = frameAllocator
      .allocate_frame()
      .expect("failed to allocate GDT frame");

   gdt::CreateAndLoadGdt(gdtFrame);

   match unsafe { kernelPageTable.identity_map(gdtFrame, PageTableFlags::PRESENT, frameAllocator) }
   {
      Ok(tlb) => tlb.flush(),
      Err(err) => panic!("failed to identity map frame {:?}: {:?}", gdtFrame, err),
   }

   // Map the framebuffer
   let fbVirtualAddress = if let Some(framebuffer) = framebuffer {
      log::info!("Mapping framebuffer");

      let fbStartFrame = PhysFrame::containing_address(framebuffer.address);
      let fbEndFrame =
         PhysFrame::containing_address(framebuffer.address + framebuffer.info.byteLength - 1u64);

      let startPage = mappingAddressPageAligned(
         config.mappings.framebuffer,
         u64::from_usize(framebuffer.info.byteLength),
         &mut usedEntries,
         "framebuffer",
      );

      for (i, frame) in PhysFrame::range_inclusive(fbStartFrame, fbEndFrame).enumerate() {
         let page = startPage + u64::from_usize(i);
         let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

         match unsafe { kernelPageTable.map_to(page, frame, flags, frameAllocator) } {
            Ok(tlb) => tlb.flush(),
            Err(err) => panic!(
               "failed to map page {:?} to frame {:?}: {:?}",
               page, frame, err
            ),
         }
      }

      let fbVirtualAddress = startPage.start_address();
      Some(fbVirtualAddress)
   } else {
      None
   };

   let ramdiskSliceLength = systemInfo.ramdiskLength;
   let ramdiskSliceStart = if let Some(address) = systemInfo.ramdiskAddress {
      let startPage = mappingAddressPageAligned(
         config.mappings.ramdiskMemory,
         systemInfo.ramdiskLength,
         &mut usedEntries,
         "ramdisk start",
      );

      let physicalAddress = PhysAddr::new(address);
      let ramdiskPhysicalStartPage: PhysFrame<Size4KiB> =
         PhysFrame::containing_address(physicalAddress);
      let ramdiskPageCount = (systemInfo.ramdiskLength - 1) / Size4KiB::SIZE;
      let ramdiskPhysicalEndPage = ramdiskPhysicalStartPage + ramdiskPageCount;

      let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
      for (i, frame) in
         PhysFrame::range_inclusive(ramdiskPhysicalStartPage, ramdiskPhysicalEndPage).enumerate()
      {
         let page = startPage + i as u64;
         match unsafe { kernelPageTable.map_to(page, frame, flags, frameAllocator) } {
            Ok(tlb) => tlb.ignore(),
            Err(err) => panic!(
               "Failed to map page {:?} to frame {:?}: {:?}",
               page, frame, err
            ),
         };
      }

      Some(startPage.start_address())
   } else {
      None
   };

   let physicalMemoryOffset = if let Some(map) = config.mappings.physicalMemory {
      log::info!("Map physical memory");

      let startFrame = PhysFrame::containing_address(PhysAddr::zero());
      let maxPhysicalAddress = frameAllocator.MaxPhysAddress();
      let endFrame: PhysFrame<Size2MiB> = PhysFrame::containing_address(maxPhysicalAddress - 1u64);

      let size = maxPhysicalAddress.as_u64();
      let alignment = Size2MiB::SIZE;
      let offset = mappingAddress(map, size, alignment, &mut usedEntries)
         .expect("start address for physical memory must be 2MiB-page-aligned");

      for frame in PhysFrame::range_inclusive(startFrame, endFrame) {
         let page = Page::containing_address(offset + frame.start_address().as_u64());
         let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

         match unsafe { kernelPageTable.map_to(page, frame, flags, frameAllocator) } {
            Ok(tlb) => tlb.ignore(),
            Err(err) => panic!(
               "failed to map page {:?} to frame {:?}: {:?}",
               page, frame, err
            ),
         };
      }

      Some(offset)
   } else {
      None
   };

   let recursiveIndex = if let Some(mapping) = config.mappings.pageRecursiveTable {
      log::info!("Map page table recursively");

      let index = match mapping {
         Mapping::Dynamic => usedEntries.GetFreeEntries(1),
         Mapping::Fixed(offset) => {
            let offset = VirtAddr::new(offset);
            let tableLevel = PageTableLevel::Four;
            if !offset.is_aligned(tableLevel.entry_address_space_alignment()) {
               panic!(
                  "Offset for recursive mapping must be properly aligned (must be \
                        a multiple of {:#x})",
                  tableLevel.entry_address_space_alignment()
               );
            }

            offset.p4_index()
         }
      };

      let entry = &mut kernelPageTable.level_4_table()[index];
      if !entry.is_unused() {
         panic!(
            "could not set up recursive mapping: index {} is already in use",
            u16::from(index),
         );
      }

      let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
      entry.set_frame(pageTables.kernelLevel4Frame, flags);

      Some(index)
   } else {
      None
   };

   return KernelMappings {
      entryPoint,
      stackTop: stackEndAddress.align_down(16u8),
      usedEntries,
      framebuffer: fbVirtualAddress,
      physicalMemoryOffset,
      recursiveIndex,
      tlsTemplate,
      kernelSliceStart,
      kernelSliceLength,
      kernelImageOffset,

      ramdiskSliceStart,
      ramdiskSliceLength,
   };
}

pub fn CreateBootInfo<I, D>(
   config: &LoaderConfig,
   bootConfig: &BootConfig,
   mut frameAllocator: LegacyFrameAllocator<I, D>,
   pageTables: &mut PageTables,
   mappings: &mut KernelMappings,
   systemInfo: SystemInfo,
) -> &'static mut BootInfo
where
   I: ExactSizeIterator<Item = D> + Clone,
   D: LegacyMemoryRegion, {
   log::info!("Allocate boot info!");

   let config = config.clone();
   let (bootInfo, memoryRegions) = {
      let bootInfoLayout = Layout::new::<BootInfo>();
      let regions = frameAllocator.Length();
      let memoryRegionsLayout = Layout::array::<MemoryRegion>(regions).unwrap();
      let (combined, memoryRegionsOffset) = bootInfoLayout.extend(memoryRegionsLayout).unwrap();

      let bootInfoAddress = mappingAddress(
         config.mappings.bootInfo,
         u64::from_usize(combined.size()),
         u64::from_usize(combined.align()),
         &mut mappings.usedEntries,
      )
      .expect("boot info address is not properly aligned");

      let memoryMapRegionsAddress = bootInfoAddress + memoryRegionsOffset;
      let memoryMapRegionsEnd = bootInfoAddress + combined.size();

      let startPage = Page::containing_address(bootInfoAddress);
      let endPage = Page::containing_address(memoryMapRegionsEnd - 1u64);
      for page in Page::range_inclusive(startPage, endPage) {
         let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
         let frame = frameAllocator
            .allocate_frame()
            .expect("frame allocation for boot info failed");

         match unsafe {
            pageTables
               .kernel
               .map_to(page, frame, flags, &mut frameAllocator)
         } {
            Ok(tlb) => tlb.flush(),
            Err(err) => panic!("failed to map page {:?}: {:?}", page, err),
         }

         // We also need access to the bootloader
         match unsafe {
            pageTables
               .bootloader
               .map_to(page, frame, flags, &mut frameAllocator)
         } {
            Ok(tlb) => tlb.flush(),
            Err(err) => panic!("failed to map page {:?}: {:?}", page, err),
         }
      }

      let bootInfo: &'static mut MaybeUninit<BootInfo> =
         unsafe { &mut *bootInfoAddress.as_mut_ptr() };

      let memoryRegions: &'static mut [MaybeUninit<MemoryRegion>] =
         unsafe { slice::from_raw_parts_mut(memoryMapRegionsAddress.as_mut_ptr(), regions) };

      (bootInfo, memoryRegions)
   };

   log::info!("Create Memory Map");

   let mut bootInfo = bootInfo.write({
      let mut info = BootInfo::new(memoryRegions.into());
      info.frameBuffer = mappings
         .framebuffer
         .map(|address| unsafe {
            PixelBuffer::new(
               address.as_u64(),
               systemInfo
                  .framebuffer
                  .expect(
                     "there should not be a mapping for the framebuffer if there is no framebuffer",
                  )
                  .info,
            )
         })
         .into();

      info.physMemoryOffset = mappings.physicalMemoryOffset.map(VirtAddr::as_u64).into();
      info.recursiveIndex = mappings.recursiveIndex.map(Into::into).into();
      info.rsdpAddress = systemInfo
         .rsdpAddress
         .map(|address| address.as_u64())
         .into();
      info.tlsTemplate = mappings.tlsTemplate.into();
      info.ramdiskAddress = mappings
         .ramdiskSliceStart
         .map(|address| address.as_u64())
         .into();
      info.ramdiskLength = mappings.ramdiskSliceLength;
      info.kernelAddress = mappings.kernelSliceStart.as_u64();
      info.kernelLength = mappings.kernelSliceLength as _;
      info.kernelImageOffset = mappings.kernelImageOffset.as_u64();
      info.testSentinel = bootConfig.testSentinel;

      info
   });

   return bootInfo;
}

pub fn SwitchToKernel(
   pageTables: PageTables,
   mappings: KernelMappings,
   bootInfo: &'static mut BootInfo,
) -> ! {
   let PageTables {
      kernelLevel4Frame, ..
   } = pageTables;

   let addresses = Addresses {
      pageTable: kernelLevel4Frame,
      stackTop: mappings.stackTop,
      entryPoint: mappings.entryPoint,
      bootInfo,
   };

   log::info!(
      "Jumping to kernel entry point at {:?}",
      addresses.entryPoint
   );

   unsafe { contextSwitch(addresses) }
}

fn mappingAddressPageAligned(
   mapping: Mapping,
   size: u64,
   usedEntries: &mut UsedLevel4Entries,
   kind: &str,
) -> Page {
   match mappingAddress(mapping, size, Size4KiB::SIZE, usedEntries) {
      Ok(address) => Page::from_start_address(address).unwrap(),
      Err(address) => panic!("{} address must be page-aligned (is `{:?}`)", kind, address),
   }
}

fn mappingAddress(
   mapping: Mapping,
   size: u64,
   alignment: u64,
   usedEntries: &mut UsedLevel4Entries,
) -> Result<VirtAddr, VirtAddr> {
   let address = match mapping {
      Mapping::Fixed(address) => VirtAddr::new(address),
      Mapping::Dynamic => usedEntries.GetFreeAddress(size, alignment),
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

unsafe fn contextSwitch(addresses: Addresses) -> ! {
   unsafe {
      asm!(
         r#"
         xor rbp, rbp
         mov cr3, {}
         mov rsp, {}
         push 0
         jmp {}
         "#,
         in(reg) addresses.pageTable.start_address().as_u64(),
         in(reg) addresses.stackTop.as_u64(),
         in(reg) addresses.entryPoint.as_u64(),
         in("rdi") addresses.bootInfo as *const _ as usize
      );
   }

   unreachable!()
}

/// The memory addresses required for the context shift.
pub struct Addresses {
   pageTable: PhysFrame,
   stackTop: VirtAddr,
   entryPoint: VirtAddr,
   bootInfo: &'static mut BootInfo,
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
   pub config: LoaderConfig,
   pub startAddress: *const u8,
   pub length: usize,
}

impl<'a> Kernel<'a> {
   pub fn parse(kernelSlice: &'a [u8]) -> Self {
      let kernelElf = ElfFile::new(kernelSlice).unwrap();

      let config = {
         let section = kernelElf.find_section_by_name(".loader-config").expect(
            "bootloader config section not found; kernel must be compiled against springboard",
         );

         let raw = section.raw_data(&kernelElf);

         LoaderConfig::deserialise(raw)
            .expect("kernel was compiled with incompatible springboard version")
      };

      return Kernel {
         elf: kernelElf,
         config,
         startAddress: kernelSlice.as_ptr(),
         length: kernelSlice.len(),
      };
   }
}

pub struct KernelMappings {
   /// The address of the kernel entry point.
   pub entryPoint: VirtAddr,
   /// The (exclusive) end address of the kernel stack.
   pub stackTop: VirtAddr,
   /// Keeps track of used entries in the L4 page table.
   /// Useful for finding free virtual memory as needed.
   pub usedEntries: UsedLevel4Entries,
   /// The start address of the framebuffer, if any.
   pub framebuffer: Option<VirtAddr>,
   /// The start address of the physical memory mapping,
   /// if enabled.
   pub physicalMemoryOffset: Option<VirtAddr>,
   /// The L4 page table index of the recursive mapping,
   /// if enabled.
   pub recursiveIndex: Option<PageTableIndex>,
   /// The thread-local storage template of the kernel
   /// executable, if it contains one.
   pub tlsTemplate: Option<TlsTemplate>,

   /// Start address of the kernel slice allocation in memory.
   pub kernelSliceStart: PhysAddr,
   /// Size of the kernel slice allocation in memory.
   pub kernelSliceLength: u64,
   /// Relocation offset of the kernel image in virtual memory.
   pub kernelImageOffset: VirtAddr,
   pub ramdiskSliceStart: Option<VirtAddr>,
   pub ramdiskSliceLength: u64,
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

use {
   crate::{
      api::info::{MemoryRegion, PixelBuffer, PixelBufferInfo, TlsTemplate},
      config::{BootConfig, LoaderConfig, Mapping},
      legacy::{LegacyFrameAllocator, LegacyMemoryRegion},
      level4::UsedLevel4Entries,
   },
   core::{alloc::Layout, arch::asm, mem::MaybeUninit, slice},
   usize_conversions::FromUsize,
   x86_64::{
      registers::control::{Cr0, Efer},
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
#[cfg(feature = "bios")]
pub mod bios;
pub mod config;
pub mod entropy;
pub mod framebuffer;
pub mod gdt;
#[cfg(feature = "uefi")]
pub mod gpt;
pub mod legacy;
pub mod level4;
pub mod loader;
pub mod logger;
#[cfg(feature = "bios")]
pub mod mbr;
pub mod serial;
#[cfg(feature = "uefi")]
pub mod uefi;

pub(crate) mod concat {
   include!(concat!(env!("OUT_DIR"), "/concat.rs"));
}

pub(crate) mod version_info {
   include!(concat!(env!("OUT_DIR"), "/version_info.rs"));
}

// EXPORTS //

pub use self::{api::info::BootInfo, config::LoaderConfig as BootloaderConfig};

// EXTERNS //

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
