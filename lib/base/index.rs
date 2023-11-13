//! Foundation library for Trident 3
#![allow(non_snake_case)]
#![crate_name="base"]
#![no_std]

// MODULES //

#[cfg(feature="allocator")]
pub mod alloc;
pub mod error;
pub mod externs;
pub mod memory;
pub mod optional;
