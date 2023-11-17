//! # Trident bootloader
//! — An EFI-compatible bootloader for the Trident system.
#![allow(non_snake_case)]
#![warn(missing_docs)]
#![deny(missing_abi, unsafe_op_in_unsafe_fn)]
#![no_main]
#![no_std]

#[entry]
fn efi_main(image: Handle, table: SystemTable<Boot>) -> Status {
   // Temporarily clone the y table for printing panics
   unsafe {
      *SYSTEM_TABLE.get() = Some(table.unsafe_clone());
   }

   let mut bootMode = BootMode::Disk;
}

fn loadConfigFile(
   image: Handle,
   table: &mut SystemTable<Boot>,
   mode: BootMode,
) -> Option<&'static mut [u8]> {
}

struct RacyCell<T>(pub UnsafeCell<T>);

impl<T> RacyCell<T> {
   pub const fn new(v: T) -> Self {
      Self(UnsafeCell::new(v))
   }
}

unsafe impl<T> Sync for RacyCell<T> {}

impl<T> Deref for RacyCell<T> {
   type Target = UnsafeCell<T>;

   fn deref(&self) -> &Self::Target {
      &self.0
   }
}

static SYSTEM_TABLE: RacyCell<Option<SystemTable<Boot>>> = RacyCell::new(None);

#[derive(Clone, Copy, Debug)]
pub enum BootMode {
   Disk,
   Tftp,
}

// MODULES //

pub mod descriptor;

extern crate base;
extern crate log;
extern crate serde_json_core as json;
extern crate uefi;
extern crate x86_64;

// IMPORTS //

use {
   core::{
      cell::UnsafeCell,
      ops::{Deref, DerefMut},
      ptr, slice,
   },
   uefi::{
      prelude::{entry, Boot, Handle, Status, SystemTable},
      proto::{
         console::gop::{GraphicsOutput, PixelFormat},
         device_path::DevicePath,
         loaded_image::LoadedImage,
         media::{
            file::{File, FileAttribute, FileInfo, FileMode},
            fs::SimpleFileSystem,
         },
         network::{
            pxe::{BaseCode, DhcpV4Packet},
            IpAddress,
         },
         ProtocolPointer,
      },
      table::boot::{
         AllocateType, MemoryType, OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol,
      },
      CStr16, CStr8,
   },
   x86_64::{
      structures::paging::{FrameAllocator, PageOffset, PageTable, PhysFrame, Size4KiB},
      PhysAddr, VirtAddr,
   },
};
