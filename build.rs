#![allow(non_snake_case)] // An annoying hack around rustfmt enforcing syntax ;—;

fn main() {
   // Set by Cargo, and our build script uses this directory for output files.
   let outDir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

   let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_TRIDENT3_MAIN_t3_main").unwrap());

   // Create an EFI-compatible boot image
   let uefiPath = outDir.join("uefi.img");
   UefiBoot::new(&kernel).create_disk_image(&uefiPath).unwrap();

   // Create a legacy BIOS-compatible boot image
   let biosPath = outDir.join("bios.img");
   BiosBoot::new(&kernel).create_disk_image(&biosPath).unwrap();

   // pass the disk image paths as env variables to the `main.rs`
   println!("cargo:rustc-env=UEFI_PATH={}", uefiPath.display());
   println!("cargo:rustc-env=BIOS_PATH={}", biosPath.display());
}

// IMPORTS //

use {
   springboard::{BiosBoot, UefiBoot},
   std::path::PathBuf,
};
