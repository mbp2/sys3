#![allow(non_snake_case)]

const KERNEL_FILE_NAME: &str = "kernel-x86_64";
const RAMDISK_FILE_NAME: &str = "ramdisk";
const CONFIG_FILE_NAME: &str = "boot.json";

/// Allows creating disk images for a specified set of files.
///
/// It can currently create `MBR` (BIOS), `GPT` (UEFI), and `TFTP` (UEFI) images.
pub struct DiskImageBuilder {
   files: BTreeMap<Cow<'static, str>, FileDataSource>,
}

impl DiskImageBuilder {
   pub fn new(kernel: PathBuf) -> Self {
      let mut object = Self::empty();
      object.SetKernel(kernel);
      return object;
   }

   pub fn empty() -> Self {
      return DiskImageBuilder{
         files: BTreeMap::new(),
      };
   }

   pub fn SetKernel(&mut self, path: PathBuf) -> &mut Self {
      return self.setFileSource(
         KERNEL_FILE_NAME.into(),
         FileDataSource::File(path)
      );
   }

   pub fn SetRamdisk(&mut self, path: PathBuf) -> &mut Self {
      return self.setFileSource(
         RAMDISK_FILE_NAME.into(),
         FileDataSource::File(path),
      );
   }

   pub fn SetBootConfig(&mut self, config: &BootConfig) -> &mut Self {
      let json = serde_json::to_vec_pretty(config)
         .expect("failed to serialise boot config");

      return self.setFileSource(
         CONFIG_FILE_NAME.into(),
         FileDataSource::Data(json)
      );
   }

   pub fn SetFileContents(&mut self, destination: String, data: Vec<u8>) -> &mut Self {
      return self.setFileSource(destination.into(), FileDataSource::Data(data));
   }

   pub fn SetFile(&mut self, destination: String, path: PathBuf) -> &mut Self {
      return self.setFileSource(destination.into(), FileDataSource::File(path));
   }

   fn setFileSource(&mut self, destination: Cow<'static, str>, source: FileDataSource) -> &mut Self {
      self.files.insert(destination, source);
      return self;
   }

   fn createFatFilesystemImage(
      &self,
      internalFiles: BTreeMap<&str, FileDataSource>
   ) -> anyhow::Result<NamedTempFile> {
      let mut localMap: BTreeMap<&str, _> = BTreeMap::new();

      for (name, source) in &self.files {
         localMap.insert(name, source);
      }

      for k in &internalFiles {
         if localMap.insert(k.0, k.1).is_some() {
            return Err(anyhow::Error::msg(format!(
               "Attempted to overwrite internal file: {}",
               k.0
            )));
         }
      }

      let out = NamedTempFile::new().context("failed to create temp file")?;
      fat::CreateFatFilesystem(localMap, out.path())
         .context("failed to create BIOS FAT filesystem")?;

      return Ok(out);
   }
}

pub fn add(left: usize, right: usize) -> usize {
   left + right
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn it_works() {
      let result = add(2, 2);
      assert_eq!(result, 4);
   }
}

// MODULES //

#[cfg(feature="bios")]
pub mod bios;
pub mod fat;
#[cfg(feature="uefi")]
pub mod gpt;
#[cfg(feature="bios")]
pub mod mbr;
pub mod source;
#[cfg(feature="uefi")]
pub mod uefi;

// IMPORTS //

use {
   crate::source::FileDataSource,
   anyhow::Context,
   std::{
      borrow::Cow,
      collections::BTreeMap,
      path::{Path, PathBuf},
   },
   tempfile::NamedTempFile,
};

// EXPORTS //

pub use springboard::config::BootConfig;

#[cfg(feature="bios")]
pub use self::bios::BiosBoot;

#[cfg(feature="uefi")]
pub use self::uefi::UefiBoot;

// EXTERNS //

extern crate alloc;
extern crate anyhow;
extern crate base;
extern crate springboard;
extern crate core;
