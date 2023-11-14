/// This structure represents the information that the bootloader passes to the kernel.
///
/// The information is passed as an argument to the entry point. The entry point function must
/// have the following signature:
///
/// ```
/// # use springboard::api::BootInfo;
/// # type _SIGNATURE =
/// extern "C" fn(boot_info: &'static mut BootInfo) -> !;
/// ```
///
/// Note that no type checking occurs for the entry point function, so be careful to
/// use the correct argument types. To ensure that the entry point function has the correct
/// signature, use the [`entry_point`] macro.
#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
pub struct BootInfo {
   /// Version of our bootloader, must match exactly with `springboard` version.
   pub apiVersion: ApiVersion,

   /// A map of the physical memory regions of the underlying machine.
   ///
   /// The bootloader queries this information from the BIOS/UEFI firmware and translates this
   /// information to Rust types. It also marks any memory regions that the bootloader uses in
   /// the memory map before passing it to the kernel. Regions marked as usable can be freely
   /// used by the kernel.
   pub memoryRegions: MemoryRegions,

   /// Information about the framebuffer for screen output if available.
   pub frameBuffer: Optional<PixelBuffer>,

   /// The virtual address at which the mapping of the physical memory starts.
   ///
   /// Physical addresses can be converted to virtual addresses by adding this offset to them.
   ///
   /// The mapping of the physical memory allows to access arbitrary physical frames. Accessing
   /// frames that are also mapped at other virtual addresses can easily break memory safety and
   /// cause undefined behavior. Only frames reported as `USABLE` by the memory map in the `BootInfo`
   /// can be safely accessed.
   ///
   /// Only available if the `map-physical-memory` config option is enabled.
   pub physMemoryOffset: Optional<u64>,

   /// The virtual address of the recursively mapped level 4 page table.
   ///
   /// Only available if the `map-page-table-recursively` config option is enabled.
   pub recursiveIndex: Optional<u16>,

   /// The address of the `RSDP` data structure, which can be use to find the ACPI tables.
   ///
   /// This field is `None` if no `RSDP` was found (for BIOS) or reported (for UEFI).
   pub rsdpAddress: Optional<u64>,

   /// The thread local storage (TLS) template of the kernel executable, if present.
   pub tlsTemplate: Optional<TlsTemplate>,

   /// Ramdisk address, if loaded
   pub ramdiskAddress: Optional<u64>,

   /// Ramdisk image size, set to 0 if addr is None
   pub ramdiskLength: u64,

   /// Physical address of the kernel ELF in memory.
   pub kernelAddress: u64,

   /// Size of the kernel ELF in memory.
   pub kernelLength: u64,

   /// Virtual address of the loaded kernel image.
   pub kernelImageOffset: u64,

   #[doc(hidden)]
   pub testSentinel: u64,
}

impl BootInfo {
   /// Create a new boot info structure with the given memory map.
   ///
   /// The other fields are initialized with default values.
   pub fn new(regions: MemoryRegions) -> Self {
      return BootInfo {
         apiVersion: ApiVersion::new_default(),
         memoryRegions: regions,
         frameBuffer: Optional::None,
         physMemoryOffset: Optional::None,
         recursiveIndex: Optional::None,
         rsdpAddress: Optional::None,
         tlsTemplate: Optional::None,
         ramdiskAddress: Optional::None,
         ramdiskLength: 0,
         kernelAddress: 0,
         kernelLength: 0,
         kernelImageOffset: 0,
         testSentinel: 0,
      };
   }
}

/// FFI-safe slice of [`MemoryRegion`] structs, semantically equivalent to
/// `&'static mut [MemoryRegion]`.
///
/// This type implements the [`Deref`][core::ops::Deref] and [`DerefMut`][core::ops::DerefMut]
/// traits, so it can be used like a `&mut [MemoryRegion]` slice. It also implements [`From`]
/// and [`Into`] for easy conversions from and to `&'static mut [MemoryRegion]`.
#[derive(Debug)]
#[repr(C)]
pub struct MemoryRegions {
   pub(crate) ptr: *mut MemoryRegion,
   pub(crate) length: usize,
}

impl ops::Deref for MemoryRegions {
   type Target = [MemoryRegion];

   fn deref(&self) -> &Self::Target {
      unsafe { slice::from_raw_parts(self.ptr, self.length) }
   }
}

impl ops::DerefMut for MemoryRegions {
   fn deref_mut(&mut self) -> &mut Self::Target {
      unsafe { slice::from_raw_parts_mut(self.ptr, self.length) }
   }
}

impl From<&'static mut [MemoryRegion]> for MemoryRegions {
   fn from(regions: &'static mut [MemoryRegion]) -> Self {
      return MemoryRegions {
         ptr: regions.as_mut_ptr(),
         length: regions.len(),
      };
   }
}

impl From<MemoryRegions> for &'static mut [MemoryRegion] {
   fn from(regions: MemoryRegions) -> &'static mut [MemoryRegion] {
      unsafe { slice::from_raw_parts_mut(regions.ptr, regions.length) }
   }
}

/// Represent a physical memory region.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct MemoryRegion {
   /// The physical start address of the memory region.
   pub start: u64,

   /// The physical end address of the memory region
   pub end: u64,

   /// The memory type of the region.
   ///
   /// Only [`Usable`][MemoryRegionKind::Usable] regions can be freely used.
   pub kind: MemoryRegionKind,
}

/// Represents the different types of memory.
#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MemoryRegionKind {
   Usable,
   Bootloader,
   UnknownUefi(u32),
   UnknownBios(u32),
}

#[repr(C)]
pub struct PixelBuffer {
   pub start: u64,
   pub info: PixelBufferInfo,
}

impl PixelBuffer {
   /// Creates a new framebuffer instance.
   ///
   /// ## Safety
   ///
   /// The given start address and info must describe a valid, accessible, and unaliased
   /// framebuffer.
   pub unsafe fn new(start: u64, info: PixelBufferInfo) -> Self {
      return PixelBuffer{ start, info };
   }

   pub fn Buffer(&self) -> &[u8] {
      unsafe { self.createBuffer() }
   }

   pub fn BufferMut(&mut self) -> &mut [u8] {
      unsafe { self.createMutBuffer() }
   }

   pub fn OwnedBuffer(self) -> &'static mut [u8] {
      unsafe { self.createMutBuffer() }
   }

   unsafe fn createBuffer<'a>(&self) -> &'a [u8] {
      unsafe { slice::from_raw_parts(self.start as *const u8, self.info.byteLength) }
   }

   unsafe fn createMutBuffer<'a>(&self) -> &'a mut [u8] {
      unsafe { slice::from_raw_parts_mut(self.start as *mut u8, self.info.byteLength) }
   }

   pub fn info(&self) -> PixelBufferInfo {
      return self.info;
   }
}

/// Describes the layout and pixel format of a framebuffer.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PixelBufferInfo {
   /// The total size of the buffer in bytes.
   pub byteLength: usize,

   /// The width of the buffer in pixels.
   pub width: usize,

   /// The height of the buffer in pixels.
   pub height: usize,

   /// The format of the pixel.
   pub pixelFormat: PixelFormat,

   /// The number of bytes-per-pixel.
   pub bbp: usize,

   /// Number of pixels between the start of a line and the start of the next.
   ///
   /// Some framebuffers use additional padding at the end of a line, so this
   /// value might be larger than `horizontal_resolution`. It is
   /// therefore recommended to use this field for calculating the start address of a line.
   pub stride: usize,
}

/// Color format of pixels in the framebuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[repr(C)]
pub enum PixelFormat {
   /// One byte red, one byte green, then one byte blue.
   ///
   /// Length may be greater than three, check [`bbp`][PixelBufferInfo::bbp] for this.
   Rgb,

   /// One byte blue, one byte green, then one byte red.
   ///
   /// Length may be greater than three, check [`bbp`][PixelBufferInfo::bbp] for this.
   Bgr,

   /// A single-line byte, representing the greyscale value.
   ///
   /// Length may be greater than one, check [`bbp`][PixelBufferInfo::bbp] for this.
   U8,

   /// Represents an unknown pixel format.
   Unknown {
      /// Bit offset of the red value
      red: u8,
      /// Bit offset of the green value
      green: u8,
      /// Bit offset of the blue value
      blue: u8,
   },
}

/// Information about the thread local storage (TLS) template.
///
/// This template can be used to set up thread local storage for threads. For
/// each thread, a new memory location of size `mem_size` must be initialized.
/// Then the first `file_size` bytes of this template needs to be copied to the
/// location. The additional `mem_size - file_size` bytes must be initialized with
/// zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct TlsTemplate {
   /// The virtual start address of the thread local storage template.
   pub start: u64,

   /// The number of data bytes in the template.
   ///
   /// Corresponds to the length of the `.tdata` section.
   pub fileSize: u64,

   /// The total number of bytes that the TLS segment should have in memory.
   ///
   /// Corresponds to the combined length of the `.tdata` and `.tbss` sections.
   pub memSize: u64,
}

// IMPORTS //

/// Check bootinfo for FFI-safety.
extern "C" fn _assert_ffi(_bootinfo: BootInfo) {}

use {
   crate::config::ApiVersion,
   base::optional::Optional,
   core::{ops, slice},
};
