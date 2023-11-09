//! Foundation library for Trident 3
#![crate_name="base"]
#![no_std]

// MODULES //

#[cfg(feature="allocator")]
pub mod alloc;
pub mod error;
pub mod externs;
