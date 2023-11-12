//! # Trident bootloader
//! — An EFI-compatible bootloader for the Trident system.
#![allow(non_snake_case)]
#![warn(missing_docs)]
#![deny(missing_abi)]
#![no_main]
#![no_std]



// MODULES //

pub mod descriptor;

extern crate base;
extern crate log;
extern crate serde_json_core as json;
extern crate uefi;
extern crate x86_64;
