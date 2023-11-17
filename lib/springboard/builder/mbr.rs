const SECTOR_SIZE: u32 = 512;

pub fn CreateMbrDisk(
   bootSectorPath: &Path,
   secondStagePath: &Path,
   bootPartitionPath: &Path,
   out: &Path,
) -> anyhow::Result<()> {
   let mut bootSector = File::open(bootSectorPath).context("failed to open boot sector")?;
   let mut mbr = MBR::read_from(&mut bootSector, SECTOR_SIZE)
      .context("failed to read master boot record")?;

   for (index, partition) in mbr.iter() {
      if !partition.is_unused() {
         anyhow::bail!("partition {index} should be unused");
      }
   }

   let mut secondStage = File::open(secondStagePath)
      .context("failed to open second stage binary")?;

   let secondStageSize = secondStage.metadata()
      .context("failed to read metadata of second stage")?
      .len();

   let secondStageStartSector = 1;

   let secondStageSectors: u32 = ((secondStageSize - 1) / u64::from(SECTOR_SIZE) + 1)
      .try_into()
      .context("size of second stage is larger than u32::MAX")?;

   mbr[1] = MBRPartitionEntry{
      boot: BOOT_ACTIVE,
      starting_lba: secondStageStartSector,
      sectors: secondStageSectors,
      // see BOOTLOADER_SECOND_STAGE_PARTITION_TYPE in `boot_sector` crate
      sys: 0x20,
      first_chs: CHS::empty(),
      last_chs: CHS::empty(),
   };

   let mut bootPartition = File::open(bootPartitionPath)
      .context("failed to open FAT boot partition")?;

   let bootPartitionStartSector = secondStageStartSector + secondStageSectors;
   let bootPartitionSize = bootPartition.metadata()
      .context("failed to read file metadata of FAT boot partition")?
      .len();

   mbr[2] = MBRPartitionEntry{
      boot: BOOT_ACTIVE,
      starting_lba: bootPartitionStartSector,
      sectors: ((bootPartitionSize - 1) / u64::from(SECTOR_SIZE) + 1)
         .try_into().context("size of FAT partition is greater than u32::MAX")?,
      // TODO: ensure this is the correct type:
      sys: 0x0c, // FAT32 w/ LBA
      first_chs: CHS::empty(),
      last_chs: CHS::empty(),
   };

   let mut disk = fs::OpenOptions::new()
      .create(true)
      .truncate(true)
      .read(true)
      .write(true)
      .open(out)
      .with_context(|| {
         format!(
            "failed to create MBR disk image at `{}`",
            out.display()
         )
      })?;

   mbr.write_into(&mut disk).context("failed to write MBR header to disk image")?;

   assert_eq!(
      disk.stream_position().context("failed to get disk image seek position")?,
      u64::from(secondStageStartSector * SECTOR_SIZE)
   );

   io::copy(&mut secondStage, &mut disk)
      .context("failed to copy second stage binary into disk image")?;

   disk.seek(SeekFrom::Start(
      (bootPartitionStartSector * SECTOR_SIZE).into()
   )).context("seek failed")?;

   io::copy(&mut bootPartition, &mut disk)
      .context("failed to copy FAT image to MBR disk image")?;

   return Ok(());
}

// IMPORTS //

use {
   anyhow::Context,
   mbrman::{BOOT_ACTIVE, CHS, MBR, MBRPartitionEntry},
   std::{
      fs::{self, File},
      io::{self, Seek, SeekFrom},
      path::Path,
   },
};
