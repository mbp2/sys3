//! Foundation runtime library for Trident 3
#![crate_name = "base"]
#![allow(nonstandard_style)]
#![warn(missing_docs, missing_abi)]
#![feature(coerce_unsized)]
#![feature(custom_test_frameworks)]
#![feature(decl_macro)]
#![feature(naked_functions)]
#![feature(unsize)]
#![reexport_test_harness_main="test_main"]
#![test_runner(crate::test::test_runner)]
#![no_std]

// MODULES //

/// TODO: document `alloc` module.
#[cfg(feature = "allocators")]
pub mod alloc;

/// Platform-specific code.
pub mod arch;

/// TODO: document `array` module.
#[cfg(feature = "allocators")]
pub mod array;

/// TODO: document `error` module.
pub mod error;

/// TODO: document `external` module.
pub mod external;

/// TODO: document `io` module.
pub mod io;

/*
/// A simple JSON parser, lexer, and utility functions.
pub mod json;
*/

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

/// System process tracking.
pub mod process;

/// [`String`][crate::string::String]: A growable UTF-8 string.
#[cfg(feature = "allocators")]
pub mod string;

/// Facilities for handling low-level system calls.
pub mod syscall;

/// Coroutine handling facilities, i.e. `async`/`.await` and task runners.
pub mod tasks;

/// Facilities for interacting with standard input/output.
pub mod terminal;

/// Testing utilities.
pub mod test;

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

extern crate alloc as std_alloc;
extern crate bitflags;
extern crate cfg_if;
extern crate conquer_once;
extern crate core;
extern crate crossbeam_queue;
extern crate lazy_static;
extern crate noto_sans_mono_bitmap;
extern crate rustversion;
extern crate spin;
extern crate spinning_top;
extern crate springboard_api;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern crate x86;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern crate x86_64;

// EXPORTS //

/// Export the [`log`](https://docs.rs/log/latest/log) crate to enable consistent logging without additional dependencies.
pub extern crate log;
