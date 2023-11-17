#[derive(Clone)]
/// Defines a data source, either a source `std::path::PathBuf`, or a vector of bytes.
pub enum FileDataSource {
   File(PathBuf),
   Data(Vec<u8>),
}

impl Debug for FileDataSource {
   fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
      match self {
         FileDataSource::File(file) => {
            f.write_fmt(format_args!("data source: File {}", file.display()))
         }

         FileDataSource::Data(data) => {
            f.write_fmt(format_args!("data source: {} raw bytes", data.len()))
         }
      }
   }
}

impl FileDataSource {
   pub fn Length(&self) -> anyhow::Result<u64> {
      return Ok(match self {
         FileDataSource::File(path) => fs::metadata(path)
            .with_context(|| format!("failed to read metadata of file `{}`", path.display()))?
            .len(),

         FileDataSource::Data(v) => v.len() as u64
      });
   }

   pub fn CopyTo(&self, target: &mut dyn io::Write) -> anyhow::Result<()> {
      match self {
         FileDataSource::File(path) => {
            io::copy(
               &mut fs::File::open(path).with_context(|| {
                  format!("failed to open `{}` for copying", path.display())
               })?,
               target,
            )?;
         }

         FileDataSource::Data(contents) => {
            let mut cursor = Cursor::new(contents);
            io::copy(&mut cursor, target)?;
         }
      };

      return Ok(());
   }
}

// IMPORTS //

use core::fmt::Write;
use {
   alloc::vec::Vec,
   anyhow::Context,
   core::fmt::{Debug, Formatter},
   std::{
      fs,
      io::{self, Cursor},
      path::PathBuf,
   },
};
