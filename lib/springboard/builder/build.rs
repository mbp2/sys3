#![allow(non_snake_case)]

const BOOTLOADER_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
   #[cfg(not(feature="uefi"))]
   async fn uefiMain() {}

   #[cfg(not(feature="bios"))]
   async fn biosMain() {}

   block_on((uefiMain(), biosMain()).join());
}

#[cfg(feature="bios")]
async fn biosMain() {
   let outDir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
   let cargo = std::env::var("CARGO").unwrap_or_else(|| "cargo".into());

   // Run the bios build commands concurrently.
   // (Cargo already uses multiple threads for building dependencies, but these
   // BIOS crates don't have enough dependencies to utilize all cores on modern
   // CPUs. So by running the build commands in parallel, we increase the number
   // of utilized cores.)
   let (biosBootSectorPath, biosStage2Path, biosStage3Path, biosStage4Path) = {
      let biosBootSectorPath = async {
         let mut cmd = Command::new(&cargo);
         cmd.arg("install").arg("springboard-legacy-bootsector");

         let localPath = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("bin").join("springboard-legacy").join("bootsector");

         if localPath.exists() {
            // Local build
            cmd.arg("--path").arg("bin/springboard-legacy/bootsector");
            println!("cargo:rerun-if-changed={}", localPath.display());
         } else {
            cmd.arg("--git").arg("https://github.com/azyklus/springboard");
         }

         cmd.arg("--locked");
         cmd.arg("--target").arg("i386-code16-bootsector.json");
         cmd.arg("--profile").arg("stage-1");
         cmd.arg("-Zbuild-std=core").arg("-Zbuild-std-features=compiler-builtins-mem");
         cmd.arg("--root").arg(outDir.clone());

         cmd.env_remove("RUSTFLAGS");
         cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
         cmd.env_remove("RUSTC_WORKSPACE_WRAPPER");

         let status = cmd.status().await
            .expect("failed to run cargo install for BIOS bootsector");

         let elfPath = if status.success() {
            let path = outDir
               .join("bin")
               .join("springboard-legacy-bootsector");

            assert!(
               path.exists(),
               "BIOS bootsector executable does not exist after build"
            );

            path
         } else {
            panic!("failed to build BIOS boot sector");
         };

         convertElfBin(elfPath).await
      };

      let biosStage2Path = async {
         let mut cmd = Command::new(&cargo);
         cmd.arg("install").arg("springboard-legacy-stage2");

         let localPath = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("bin")
            .join("springboard-legacy")
            .join("stage2");

         if localPath.exists() {
            // Local build
            cmd.arg("--path").arg(&localPath);
            println!("cargo:rerun-if-changed={}", localPath.display());
            println!(
               "cargo:rerun-if-changed={}",
               localPath.with_file_name("common").display()
            );
         } else {
            // Fetch from git
            cmd.arg("--git").arg("https://github.com/azyklus/springboard");
         }

         cmd.arg("--locked");
         cmd.arg("--target").arg("i386-code16-stage2.json");
         cmd.arg("--profile").arg("stage-2");
         cmd.arg("-Zbuild-std=core").arg("-Zbuild-std-features=compiler-builtins-mem");
         cmd.arg("--root").arg(outDir);

         cmd.env_remove("RUSTFLAGS");
         cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
         cmd.env_remove("RUSTC_WORKSPACE_WRAPPER"); // used by clippy

         let status = cmd.status().await
            .expect("failed to run cargo install for bios second stage");

         let elfPath = if status.success() {
            let path = outDir.join("bin").join("springboard-legacy-stage2");
            assert!(
               path.exists(),
               "BIOS second stage executable does not exist after building"
            );

            path
         } else {
            panic!("failed to build BIOS second stage");
         };

         convertElfBin(elfPath).await
      };

      let biosStage3Path = async {
         let mut cmd = Command::new(&cargo);
         cmd.arg("install").arg("springboard-legacy-stage3");

         let localPath = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("bin")
            .join("springboard-legacy")
            .join("stage3");

         if localPath.exists() {
            // Local build
            cmd.arg("--path").arg(&localPath);
            println!("cargo:rerun-if-changed={}", localPath.display());
         } else {
            // Fetch from git
            cmd.arg("--git").arg("https://github.com/azyklus/springboard");
         }

         cmd.arg("--locked");
         cmd.arg("--target").arg("i686-stage3.json");
         cmd.arg("--profile").arg("stage-3");
         cmd.arg("-Zbuild-std=core").arg("-Zbuild-std-features=compiler-builtins-mem");
         cmd.arg("--root").arg(&outDir);

         cmd.env_remove("RUSTFLAGS");
         cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
         cmd.env_remove("RUSTC_WORKSPACE_WRAPPER"); // used by clippy

         let status = cmd.status().await
            .expect("failed to run cargo install for BIOS stage3");

         let elfPath = if status.success() {
            let path = outDir.join("bin").join("springboard-legacy-stage3");
            assert!(
               path.exists(),
               "bios stage-3 executable does not exist after building"
            );

            path
         } else {
            panic!("failed to build bios stage-3");
         };

         convertElfBin(elfPath).await
      };

      let biosStage4Path = async {
         let mut cmd = Command::new(&cargo);
         cmd.arg("install").arg("springboard-legacy-stage4");

         let localPath = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("bin")
            .join("springboard-legacy")
            .join("stage4");

         if localPath.exists() {
            // Local build
            cmd.arg("--path").arg(&localPath);
            println!("cargo:rerun-if-changed={}", localPath.display());
         } else {
            // Fetch from git
            cmd.arg("--git").arg("https://github.com/azyklus/springboard");
         }

         cmd.arg("--locked");
         cmd.arg("--target").arg("x86_64-stage4.json");
         cmd.arg("--profile").arg("stage-4");
         cmd.arg("-Zbuild-std=core").arg("-Zbuild-std-features=compiler-builtins-mem");
         cmd.arg("--root").arg(&outDir);

         cmd.env_remove("RUSTFLAGS");
         cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
         cmd.env_remove("RUSTC_WORKSPACE_WRAPPER"); // used by clippy

         let status = cmd.status().await
            .expect("failed to run cargo install for BIOS stage-4");

         let elfPath = if status.success() {
            let path = outDir.join("bin").join("springboard-legacy-stage4");
            assert!(
               path.exists(),
               "BIOS stage-4 executable does not exist after building"
            );
            path
         } else {
            panic!("failed to build BIOS stage-4");
         };

         convertElfBin(elfPath).await
      };

      (biosBootSectorPath, biosStage2Path, biosStage3Path, biosStage4Path).join().await
   };

   // dummy implementations because docsrs builds have no network access
   #[cfg(docsrs_dummy_build)]
   let (bios_boot_sector_path, bios_stage_2_path, bios_stage_3_path, bios_stage_4_path) = (
      PathBuf::new(),
      PathBuf::new(),
      PathBuf::new(),
      PathBuf::new(),
   );

   println!(
      "cargo:rustc-env=BIOS_BOOT_SECTOR_PATH={}",
      biosBootSectorPath.display()
   );

   println!(
      "cargo:rustc-env=BIOS_STAGE_2_PATH={}",
      biosStage2Path.display()
   );

   println!(
      "cargo:rustc-env=BIOS_STAGE_3_PATH={}",
      biosStage3Path.display()
   );

   println!(
      "cargo:rustc-env=BIOS_STAGE_4_PATH={}",
      biosStage4Path.display()
   );
}

#[cfg(feature="uefi")]
async fn uefiMain() {
   let outDir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

   #[cfg(docsrs_dummy_build)]
   let uefiPath = PathBuf::new();

   #[cfg(not(docsrs_dummy_build))]
   let uefiPath = {
      let cargo = std::env::var("CARGO").unwrap_or_else(|| "cargo".into());
      let mut cmd = Command::new(cargo);
      cmd.arg("install").arg("springboard-efi");

      let localPath = Path::new(env!("CARGO_MANIFEST_DIR"))
         .join("bin")
         .join("springboard-efi");

      if localPath.exists() {
         // Local build
         cmd.arg("--path").arg(localPath);
         println!("cargo:rerun-if-changed={}", localPath.display());
         println!("cargo:rerun-if-changed=base")
      } else {
         // Fetch from git
         cmd.arg("--git").arg("https://github.com/azyklus/springboard");
      }

      cmd.arg("--locked");
      cmd.arg("--target").arg("x86_64-unknown-uefi");
      cmd.arg("-Zbuild-std=core").arg("-Zbuild-std-features=compiler-builtins-mem");
      cmd.arg("--root").arg(outDir);
      cmd.env_remove("RUSTFLAGS");
      cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");

      let status = cmd.status().await
         .expect("failed to run cargo install for EFI bootloader");

      if status.success() {
         let path = outDir.join("bin").join("springboard-efi.efi");
         assert!(
            path.exists(),
            "EFI bootloader executable does not exist after build"
         );

         path
      } else {
         panic!("failed to build EFI bootloader");
      }
   };

   println!(
      "cargo:rustc-env=UEFI_BOOTLOADER_PATH={}",
      uefiPath.display()
   );
}

#[cfg(not(docsrs_dummy_build))]
#[cfg(feature="bios")]
async fn convertElfBin(elf: PathBuf) -> PathBuf {
   let flatBinaryPath = elf.with_extension("bin");

   let llvmTools = llvm_tools::LlvmTools::new().expect("failed to get llvm tools");
   let objcopy = llvmTools
      .tool(&llvm_tools::exe("llvm-objcopy"))
      .expect("LlvmObjcopyNotFound");

   // convert first stage to binary
   let mut cmd = Command::new(objcopy);
   cmd.arg("-I").arg("elf64-x86-64");
   cmd.arg("-O").arg("binary");
   cmd.arg("--binary-architecture=i386:x86-64");
   cmd.arg(&elf);
   cmd.arg(&flatBinaryPath);
   let output = cmd
      .output()
      .await
      .expect("failed to execute llvm-objcopy command");
   if !output.status.success() {
      panic!(
         "objcopy failed: {}",
         String::from_utf8_lossy(&output.stderr)
      );
   }

   return flatBinaryPath;
}

// IMPORTS //

use {
   async_process::Command,
   futures::executor::block_on,
   futures_concurrency::future::Join,
   std::path::{Path, PathBuf},
};
