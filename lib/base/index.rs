//! Foundation library for Trident 3
#![crate_name = "base"]
#![feature(decl_macro, coerce_unsized, unsize)]
#![no_std]

// MODULES //

#[cfg(feature = "allocators")]
pub mod alloc;
#[cfg(feature = "allocators")]
pub mod array;
pub mod error;
pub mod externs;
pub mod math;
pub mod memory;
pub mod optional;
pub mod pointer;
#[cfg(feature = "allocators")]
pub mod string;

// IMPORTS //

#[cfg(any(feature="std-allocators", not(feature="allocators")))]
extern crate alloc as std_alloc;
extern crate core;
