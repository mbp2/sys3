/// Create disk images for booting on legacy BIOS systems.
pub struct BiosBoot {
   pub builder: DiskImageBuilder,
}

impl BiosBoot {
   /// Start creating a disk image for the given bootloader ELF executable.
   pub fn new(kernelPath: &Path) -> Self {
      return BiosBoot{
         builder: DiskImageBuilder::new(kernelPath.to_owned()),
      };
   }

   /// Add a ramdisk file to the image.
   pub fn SetRamdisk(&mut self, ramdiskPath: &Path) -> &mut Self {
      self.builder.SetRamdisk(ramdiskPath.to_path_buf());
      return self;
   }

   /// Create a configuration file that configures the runtime behaviour of the bootloader.
   pub fn SetBootConfig(&mut self, config: &BootConfig) -> &mut Self {
      self.builder.SetBootConfig(config);
      return self;
   }

   /// Create a bootable BIOS disk image at the given path.
   pub fn CreateDiskImage(&self, out: &Path) -> anyhow::Result<()> {
      return self.builder.CreateBiosImage(out);
   }
}

// IMPORTS //

use {
   crate::DiskImageBuilder,
   springboard::config::BootConfig,
   std::path::Path,
};
