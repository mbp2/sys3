/// Taken from https://github.com/rust-lang/rust/blob/e100ec5bc7cd768ec17d75448b29c9ab4a39272b/library/core/src/slice/mod.rs#L1673-L1677
///
/// TODO replace with `split_array` feature in stdlib as soon as it's stabilized,
/// see https://github.com/rust-lang/rust/issues/90091
fn splitArrayRef<const N: usize, T>(slice: &[T]) -> (&[T; N], &[T]) {
   let (a, b) = slice.split_at(N);
   // SAFETY: a points to [T; N]? Yes it's [T] of length N (checked by split_at)
   unsafe { (&*(a.as_ptr() as *const [T; N]), b) }
}

/// Allows configuring the bootloader behavior.
///
/// TODO: describe use together with `entry_point` macro
/// TODO: example
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub struct LoaderConfig {
   /// The version of the bootloader API.
   ///
   /// Automatically generated from the crate version. Checked on deserialization to
   /// ensure that the kernel and bootloader use the same API version, i.e. the same config
   /// and boot info format.
   pub(crate) version: ApiVersion,

   /// Configuration for (optional) page table mappings created by the bootloader.
   pub mappings: Mappings,

   /// The size of the stack that the bootloader should allocate for the kernel (in bytes).
   ///
   /// The bootloader starts the kernel with a valid stack pointer. This setting defines
   /// the stack size that the bootloader should allocate and map.
   ///
   /// The stack is created with a additional guard page, so a stack overflow will lead to
   /// a page fault.
   pub kernelStackSize: u64,
}

impl LoaderConfig {
   pub(crate) const UUID: [u8; 16] = [
      0x74, 0x3C, 0xA9, 0x61, 0x09, 0x36, 0x46, 0xA0, 0xBB, 0x55, 0x5C, 0x15, 0x89, 0x15, 0x25,
      0x3D,
   ];
   
   #[doc(hidden)]
   pub const SERIALIZED_LEN: usize = 124;
   
   pub const fn new() -> Self {
      return LoaderConfig{
         kernelStackSize: 80 * 1024,
         mappings: Mappings::new(),
         version: ApiVersion::new_default(),
      };
   }

   pub const fn Serialise(&self) -> [u8; Self::SERIALIZED_LEN] {
      let Self{
         kernelStackSize,
         mappings,
         version,
      } = self;

      let ApiVersion{
         major,
         minor,
         patch,
         preRelease,
      } = version;

      let Mappings{
         kernelStack,
         bootInfo,
         frameBuffer,
         physicalMemory,
         pageRecursiveTable,
         aslr,
         dynamicRangeStart,
         dynamicRangeEnd,
         ramdiskMemory,
      } = mappings;

      let FrameBuffer{
         minHeight,
         minWidth,
      } = frameBuffer;

      let version = {
         let one = concat_2_2(major.to_le_bytes(), minor.to_le_bytes());
         let two = concat_2_1(patch.to_le_bytes(), [*preRelease as u8]);
         concat_4_3(one, two)
      };

      let buf = concat_16_7(Self::UUID, version);
      let buf = concat_23_8(buf, kernelStackSize.to_le_bytes());

      let buf = concat_31_9(buf, kernelStack.Serialise());
      let buf = concat_40_9(buf, bootInfo.Serialise());
      let buf = concat_49_9(buf, frameBuffer.Serialise());

      let buf = concat_58_10(
         buf,
         match physicalMemory {
            Option::None => [0; 10],
            Option::Some(m) => concat_1_9([1], m.Serialise()),
         },
      );

      let buf = concat_68_10(
         buf,
         match pageRecursiveTable {
            Option::None => [0; 10],
            Option::Some(m) => concat_1_9([1], m.Serialise()),
         },
      );

      let buf = concat_78_1(buf, [(*aslr) as u8]);

      let buf = concat_79_9(buf, match dynamicRangeStart {
         Option::None => [0; 9],
         Option::Some(address) => concat_1_8([1], address.to_le_bytes()),
      });

      let buf = concat_88_9(buf, match dynamicRangeEnd {
         Option::None => [0; 9],
         Option::Some(address) => concat_1_8([1], address.to_le_bytes()),
      });

      let buf = concat_97_9(buf, ramdiskMemory.Serialise());

      let buf = concat_106_9(buf, match minHeight {
         Option::None => [0; 9],
         Option::Some(address) => concat_1_8([1], address.to_le_bytes()),
      });

      concat_115_9(buf, match minWidth {
         Option::None => [0; 9],
         Option::Some(address) => concat_1_8([1], address.to_le_bytes()),
      })
   }

   pub fn Deserialise(serialised: &[u8]) -> Result<Self, &'static str> {
      if serialised.len() != Self::SERIALIZED_LEN {
         return Err("invalid len");
      }

      let s = serialised;

      let (version, s) = {
         let (&major, s) = splitArrayRef(s);
         let (&minor, s) = splitArrayRef(s);
         let (&patch, s) = splitArrayRef(s);
         let (&pre, s) = splitArrayRef(s);

         let pre = match pre {
            [0] => false,
            [1] => true,
         };

         let version = ApiVersion{
            major: u16::from_le_bytes(major),
            minor: u16::from_le_bytes(minor),
            patch: u16::from_le_bytes(patch),
            preRelease: pre,
         };

         (&version, s)
      };


      let (&kernelStackSize, s) = splitArrayRef(s);

      let (mappings, s) = {
         let (&kernelStack, s) = splitArrayRef(s);
         let (&bootInfo, s) = splitArrayRef(s);
         let (&frameBuffer, s) = splitArrayRef(s);
         let (&physicalMemorySome, s) = splitArrayRef(s);
         let (&physicalMemory, s) = splitArrayRef(s);
         let (&prtSome, s) = splitArrayRef(s);
         let (&prt, s) = splitArrayRef(s);
         let (&[aslr], s) = splitArrayRef(s);
         let (&drsSome, s) = splitArrayRef(s);
         let (&drs, s) = splitArrayRef(s);
         let (&dreSome, s) = splitArrayRef(s);
         let (&dre, s) = splitArrayRef(s);
         let (&rdm, s) = splitArrayRef(s);

         let mappings = Mappings{
            kernelStack: Mapping::Deserialise(&kernelStack)?,
            bootInfo: Mapping::Deserialise(&bootInfo)?,
            frameBuffer: Mapping::Deserialise(&frameBuffer)?,
            physicalMemory: match physicalMemorySome {
               [0] if physicalMemory == [0; 9] => Option::None,
               [1] => Option::Some(Mapping::Deserialise(&physicalMemory)?),
               _ => return Err("invalid physical memory value"),
            },
            pageRecursiveTable: match prtSome {
               [0] if prt == [0; 9] => Option::None,
               [1] => Option::Some(Mapping::Deserialise(&prt)?),
               _ => return Err("invalid prt value"),
            },
            aslr: match aslr {
               1 => true,
               0 => false,
               _ => return Err("invalid aslr value"),
            },
            dynamicRangeStart: match drsSome {
               [0] if drs == [0; 8] => Option::None,
               [1] => Option::Some(u64::from_le_bytes(drs)),
               _ => return Err("invalid dynamic range start value"),
            },
            dynamicRangeEnd: match dreSome {
               [0] if dre == [0; 8] => Option::None,
               [1] => Option::Some(u64::from_le_bytes(dre)),
               _ => return Err("invalid dynamic range end value"),
            },
            ramdiskMemory: Mapping::Deserialise(&rdm)?,
         };

         (mappings, s)
      };

      if !s.is_empty() {
         return Err("unexpected rest");
      }

      return Ok(LoaderConfig{
         version,
         mappings,
         kernelStackSize
      });
   }
}

impl Default for LoaderConfig {
   fn default() -> Self {
      LoaderConfig::new()
   }
}

pub struct ApiVersion {
   /// Bootloader version. (major).
   pub major: u16,
   /// Bootloader version. (minor).
   pub minor: u16,
   /// Bootloader version. (patch).
   pub patch: u16,

   /// Whether or not our bootloader version is a pre-release.
   ///
   /// We will not store the whole pre-release version number because it could be arbitrarily long.
   pub preRelease: bool,
}

impl ApiVersion {
   pub(crate) const fn new_default() -> Self {
      return ApiVersion {
         major: version_info::VERSION_MAJOR,
         minor: version_info::VERSION_MINOR,
         patch: version_info::VERSION_PATCH,
         preRelease: version_info::VERSION_PRE,
      };
   }

   pub fn IsPreRelease(&self) -> bool {
      return self.preRelease;
   }
}

impl Default for ApiVersion {
   fn default() -> Self {
      return ApiVersion::new_default();
   }
}

/// Allows to configure the virtual memory mappings created by the bootloader.
pub struct Mappings {
   /// Configures how the kernel stack should be mapped.
   ///
   /// If a fixed address is set, it must be page aligned.
   ///
   /// Note that the first page of the kernel stack is intentionally left unmapped
   /// to act as a guard page. This ensures that a page fault occurs on a stack
   /// overflow. For example, setting the kernel stack address to
   /// `FixedAddress(0xf_0000_0000)` will result in a guard page at address
   /// `0xf_0000_0000` and the kernel stack starting at address `0xf_0000_1000`.
   pub kernelStack: Mapping,
   
   /// Specifies where the [`crate::BootInfo`] struct should be placed in virtual memory.
   pub bootInfo: Mapping,
   
   /// Specifies the mapping of the frame buffer memory region.
   pub frameBuffer: Mapping,

   /// The bootloader supports to map the whole physical memory into the virtual address
   /// space at some offset. This is useful for accessing and modifying the page tables set
   /// up by the bootloader.
   ///
   /// Defaults to `None`, i.e. no mapping of the physical memory.
   pub physicalMemory: Option<Mapping>,
   
   /// As an alternative to mapping the whole physical memory (see [`Self::physical_memory`]),
   /// the bootloader also has support for setting up a
   /// [recursive level 4 page table](https://os.phil-opp.com/paging-implementation/#recursive-page-tables).
   ///
   /// Defaults to `None`, i.e. no recursive mapping.
   pub pageRecursiveTable: Option<Mapping>,

   /// Whether to randomize non-statically configured addresses.
   /// The kernel base address will be randomized when it's compiled as
   /// a position independent executable.
   ///
   /// Defaults to `false`.
   pub aslr: bool,
   
   /// The lowest virtual address for dynamic addresses.
   ///
   /// Defaults to `0`.
   pub dynamicRangeStart: Option<u64>,
   
   /// The highest virtual address for dynamic addresses.
   ///
   /// Defaults to `0xffff_ffff_ffff_f000`.
   pub dynamicRangeEnd: Option<u64>,

   /// Virtual address to map ramdisk image, if present on disk
   /// Defaults to dynamic
   pub ramdiskMemory: Mapping,
}

impl Mappings {
   /// Creates a new mapping configuration with dynamic mapping for kernel, boot info and
   /// frame buffer. Neither physical memory mapping nor recursive page table creation are
   /// enabled.
   pub const fn new() -> Self {
      return Mappings{
         kernelStack: Mapping::new(),
         bootInfo: Mapping::new(),
         frameBuffer: Mapping::new(),
         physicalMemory: Option::None,
         pageRecursiveTable: Option::None,
         aslr: false,
         dynamicRangeStart: Option::None,
         dynamicRangeEnd: Option::None,
         ramdiskMemory: Mapping::new(),
      };
   }
}

/// Specifies how the bootloader should map a memory region into the virtual address space.
pub enum Mapping {
   /// Look for an unused virtual memory region at runtime.
   Dynamic,

   /// Try to map the region at the given virtual address.
   ///
   /// The given virtual address must be page-aligned.
   ///
   /// This setting can lead to runtime boot errors if the given address is not aligned,
   /// already in use, or invalid for other reasons.
   Fixed(u64),
}

impl Mapping {
   /// Creates a new [`Mapping::Dynamic`].
   /// 
   /// This function is basically identical to [`Default::default`], with the only difference
   /// being that this is a `const fn`.
   pub const fn new() -> Self {
      return Mapping::Dynamic;
   }

   pub const fn Serialise(&self) -> [u8; 9] {
      match self {
         Mapping::Dynamic => [0; 9],
         Mapping::Fixed(address) => concat_1_8([1], address.to_le_bytes()),
      }
   }

   pub fn Deserialise(serialised: &[u8; 9]) -> Result<Self, &'static str> {
      let (&variant, s) = splitArrayRef(serialised);
      let (&address, s) = splitArrayRef(s);

      if !s.is_empty() {
         return Err("invalid mapping format");
      }

      match variant {
         [0] if address == [0; 8] => Ok(Mapping::Dynamic),
         [1] => Ok(Mapping::Fixed(u64::from_le_bytes(address))),
         _ => Err("invalid mapping value")
      }
   }
}

/// Configures the boot behavior of the bootloader.
pub struct BootConfig {
   /// Configuration for the frame buffer setup.
   pub frameBuffer: FrameBuffer,

   /// The minimum log level that is printed to the screen during boot.
   ///
   /// The default is [`LevelFilter::Trace`].
   pub logLevel: LevelFilter,

   /// Whether the bootloader should print log messages to the framebuffer during boot.
   ///
   /// Enabled by default.
   pub frameBufferLogging: bool,

   /// Whether the bootloader should print log messages to the serial port during boot.
   ///
   /// Enabled by default.
   pub serialLogging: bool,

   #[doc(hidden)]
   pub testSentinel: u64,
}

impl Default for BootConfig {
   fn default() -> Self {
      return Self{
         frameBuffer: Default::default(),
         logLevel: Default::default(),
         frameBufferLogging: true,
         serialLogging: true,
         testSentinel: 0,
      };
   }
}

/// Configuration for frame buffer used for graphical output
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub struct FrameBuffer {
   pub minHeight: Option<u64>,
   pub minWidth: Option<u64>,
}

impl FrameBuffer {
   /// Create a new framebuffer config with no requirements.
   pub const fn new() -> Self {
      return FrameBuffer{
         minHeight: Option::None,
         minWidth: Option::None,
      };
   }
}

/// An enum representing the available verbosity level filters of the logger.
///
/// Based on
/// <https://github.com/rust-lang/log/blob/dc32ab999f52805d5ce579b526bd9d9684c38d1a/src/lib.rs#L552-565>
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LevelFilter {
   /// A level lower than all log levels.
   Off,
   /// Corresponds to the `Error` log level.
   Error,
   /// Corresponds to the `Warn` log level.
   Warn,
   /// Corresponds to the `Info` log level.
   Info,
   /// Corresponds to the `Debug` log level.
   Debug,
   /// Corresponds to the `Trace` log level.
   Trace,
}

impl Default for LevelFilter {
   fn default() -> Self {
      Self::Trace
   }
}

use {
   crate::{concat::*, version_info},
   serde::{Deserialize, Serialize}
};
