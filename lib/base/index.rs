//! Foundation library for Trident 3
#![allow(non_snake_case)]
#![crate_name = "base"]
#![no_std]

// MODULES //

#[cfg(feature = "allocators")]
pub mod alloc;
pub mod error;
pub mod externs;
pub mod math;
pub mod memory;
pub mod optional;

// IMPORTS //

#[cfg(not(feature="allocators"))]
extern crate alloc as std_alloc;
extern crate core;
