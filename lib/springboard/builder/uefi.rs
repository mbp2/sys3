pub struct UefiBoot {
   pub builder: DiskImageBuilder,
}

impl UefiBoot {
   /// Start creating a disk image for the given bootloader ELF executable.
   pub fn new(kernelPath: &Path) -> Self {
      return UefiBoot{
         builder: DiskImageBuilder::new(kernelPath.to_owned()),
      };
   }

   /// Add a ramdisk file to the image.
   pub fn SetRamdisk(&mut self, ramdiskPath: &Path) -> &mut Self {
      self.builder.SetRamdisk(ramdiskPath.to_owned());
      return self;
   }

   /// Creates a configuration file (boot.json) that configures the runtime behavior of the bootloader.
   pub fn SetBootConfig(&mut self, config: &BootConfig) -> &mut Self {
      self.builder.SetBootConfig(config);
      return self;
   }

   /// Create a bootable UEFI disk image at the given path.
   pub fn CreateDiskImage(&self, out: &Path) -> anyhow::Result<()> {
      return self.builder.CreateUefiImage(out);
   }

   /// Prepare a folder for use with booting over UEFI_PXE.
   ///
   /// This places the bootloader executable under the path "bootloader". The
   /// DHCP server should set the filename option to that path, otherwise the
   /// bootloader won't be found.
   pub fn CreatePxeTftpFolder(&self, out: &Path) -> anyhow::Result<()> {
      return self.builder.CreateUefiTftpFolder(out);
   }
}

// IMPORTS //

use {
   crate::DiskImageBuilder,
   springboard::config::BootConfig,
   std::path::Path,
};
