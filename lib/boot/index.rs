//! # Trident bootloader
//! — A bootloader for the Trident system.
#![allow(non_snake_case)]
#![warn(missing_docs)]
#![deny(missing_abi)]
#![no_std]


// MODULES //

pub mod api;
pub mod config;
pub mod entropy;
pub mod framebuffer;

extern crate base;
extern crate conquer_once;
extern crate log;
extern crate rand;
extern crate rand_hc;
extern crate serde;
extern crate spinning_top;
extern crate noto_sans_mono_bitmap;
extern crate uart_16550;
extern crate usize_conversions;
