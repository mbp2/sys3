pub fn CreateFatFilesystem(files: BTreeMap<&str, &FileDataSource>, out: &Path) -> anyhow::Result<()> {
   const MB: u64 = 1024 * 1024;

   // Calculate needed size.
   let mut neededSize = 0;
   for source in files.values() {
      neededSize += source.len()?;
   }

   let fatFile = fs::OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .truncate(true)
      .open(out)
      .unwrap();

   let fatSizePaddedRounded = ((neededSize + 1024 * 64 - 1) / MB + 1) * MB + MB;
   fatFile.set_len(fatSizePaddedRounded).unwrap();

   // Our filesystem label:
   let mut label = *b"TS_TRIDENT3";

   // This __should__ always be a file, but maybe not.
   // Should we allow the caller to set the volume label instead?
   if let Some(FileDataSource::File(path)) = files.get(KERNEL_FILE_NAME) {
      if let Some(name) = path.file_stem() {
         let converted = name.to_string_lossy();
         let name = converted.as_bytes();
         let mut newLabel = [0u8; 11];
         let name = &name[..usize::min(newLabel.len(), name.len())];
         let slice = &mut newLabel[..name.len()];
         slice.copy_from_slice(name);
         label = newLabel;
      }
   }

   // Format the filesystem and open it:
   let fmtOptions = fatfs::FormatVolumeOptions::new().volume_label(label);
   fatfs::format_volume(&fatFile, fmtOptions).context("failed to format FAT file")?;
   let filesystem = fatfs::FileSystem::new(
      &fatFile,
      fatfs::FsOptions::new()
   ).context("Failed to open FAT filesystem of UEFI FAT file")?;
   let root = filesystem.root_dir();

   // Copy files to the filesystem:
   return AddFilesToImage(&root, files);
}

pub fn AddFilesToImage(
   root: &Dir<&File>,
   files: BTreeMap<&str, &FileDataSource>,
) -> anyhow::Result<()> {
   for (targetPathRaw, source) in files {
      let targetPath = Path::new(targetPathRaw);
      // Create parent directories:
      let ancestors: Vec<_> = targetPath.ancestors().skip(1).collect();
      for ancestor in ancestors.into_iter().rev().skip(1) {
         root
            .create_dir(&ancestor.display().to_string())
            .with_context(|| {
               format!("failed to create directory `{}` on FAT filesystem", ancestor.display())
            })?;
      }

      let mut newFile = root.create_file(targetPathRaw)
         .with_context(|| format!("failed to create file at `{}`", targetPath.display()))?;

      newFile.truncate().unwrap();

      source.CopyTo(&mut newFile).with_context(|| {
         format!(
            "failed to copy source data `{:?}` to file at `{}`",
            source, targetPath.display()
         )
      })?;
   }

   return Ok(());
}

// IMPORTS //

use {
   crate::{KERNEL_FILE_NAME, FileDataSource},
   anyhow::Context,
   fatfs::Dir,
   std::{
      collections::BTreeMap,
      fs::{self, File},
      path::Path,
   },
};
