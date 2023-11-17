pub fn CreateGptDisk(fatImage: &Path, out: &Path) -> anyhow::Result<()> {
   // Create new file:
   let mut disk = fs::OpenOptions::new()
      .create(true)
      .truncate(true)
      .read(true)
      .write(true)
      .open(out)
      .with_context(|| format!("failed to create GPT file at `{}`", out.display()))?;

   // Set file size:
   let partSize: u64 = fs::metadata(fatImage)
      .context("failed to read metadata of FAT image")?
      .len();

   let diskSize: u64 = partSize + 1024 * 64;
   disk.set_len(diskSize).context("failed to set GPT image file length")?;

   // create a protective MBR at LBA0 so that disk is not considered
   // unformatted on BIOS systems
   let mbr = gpt::mbr::ProtectiveMBR::with_lb_size(
      u32::try_from((diskSize / 512) - 1).unwrap_or(0xFF_FF_FF_FF)
   );

   mbr.overwrite_lba0(&mut disk).context("failed to write protective MBR")?;

   let blockSize = gpt::disk::LogicalBlockSize::Lb512;
   let mut gpt = gpt::GptConfig::new()
      .writable(true)
      .initialized(false)
      .logical_block_size(blockSize)
      .create_from_device(Box::new(&mut disk), None)
      .context("failed to create GPT structure in file")?;

   gpt.update_partitions(Default::default()).context("failed to update GPT partitions")?;

   let partID = gpt.add_partition(
      "boot",
      partSize,
      gpt::partition_types::EFI,
      0,
      None,
   ).context("failed to add boot EFI partition")?;

   let partition = gpt.partitions().get(&partID)
      .context("failed to open boot partition after creation")?;

   let startOffset = partition
      .bytes_start(blockSize)
      .context("failed to get start offset of boot partition")?;

   gpt.write().context("failed to write out GPT changes")?;

   disk.seek(io::SeekFrom::Start(startOffset)).context("failed to seek to start offset")?;

   io::copy(
      &mut File::open(fatImage).context("failed to open FAT image")?,
      &mut disk,
   ).context("failed to copy FAT image to GPT disk")?;

   return Ok(());
}

// IMPORTS //

use {
   anyhow::Context,
   std::{
      fs::{self, File},
      io::{self, Seek},
      path::Path,
   },
};
