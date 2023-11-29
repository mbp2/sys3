//! Foundation library for Trident 3
#![crate_name = "base"]
#![feature(decl_macro, coerce_unsized, unsize)]
#![warn(missing_docs, missing_abi)]
#![no_std]

// MODULES //

/// TODO: document `alloc` module.
#[cfg(feature = "allocators")]
pub mod alloc;

/// TODO: document `array` module.
#[cfg(feature = "allocators")]
pub mod array;

/// TODO: document `error` module.
pub mod error;

/// TODO: document `external` module.
pub mod externs;

/// TODO: document `io` module.
pub mod io;

/// TODO: document `math` module.
pub mod math;

/// TODO: document `memory` module.
pub mod memory;

/// Implements an FFI-safe [`Optional`][crate::optional::Optional] type, equivalent to
/// [`Option`][core::option::Option] in Standard Rust.
pub mod optional;

/// Various smart pointer implementations.
///
/// [`Unique`][crate::pointer::unique::Unique] is similar to `std::unique_ptr` in C++,
/// whereas [`Shared`][crate::pointer::shared::Shared] is similar to `std::shared_ptr` in C++ or
/// [`Arc`][alloc::sync::Arc].
pub mod pointer;

/// [`String`][crate::string::String]: A growable UTF-8 string.
#[cfg(feature = "allocators")]
pub mod string;

/// Low-level system calls.
pub mod syscall;

/// Facilities for interacting with standard input/output.
pub mod terminal;

/// A pair of UART (universal asynchronous receiver-transmitter) implementations, one memory-mapped,
/// and the other mapped to serial hardware.
///
/// [`SerialPort`][crate::uart::SerialPort]: Serial hardware-mapped UART, useful alongside pixel-buffers
/// for printing visual output.
///
/// [`MmioPort`][crate::uart::MmioPort]: A virtual, memory-mapped UART useful for the reasons above
/// on hardware that does not support physical UART hardware.
pub mod uart;

// IMPORTS //

#[cfg(any(feature="std-allocators", not(feature="allocators")))]
extern crate alloc as std_alloc;
extern crate bitflags;
extern crate cfg_if;
extern crate conquer_once;
extern crate core;
extern crate lazy_static;
extern crate log;
extern crate rustversion;
extern crate spin;
extern crate spinning_top;
extern crate springboard_api;
