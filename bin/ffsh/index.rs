// The Fast-Forward Shell is meant to provide a very simple and clean interface over the barebones
// SYSTEM-3. With this shell comes great power over the system, and with great power comes great
// responsibility--as they say.
//
// HAVE FUN!
// - Az

#![no_std]
#![no_main]
#![deny(clippy::all)]
#![allow(non_snake_case)]

#[main]
#[doc(hidden)]
fn Main() -> error::Result<()> {
   return Ok(());
}


// MODULES ////////////////////////////////////////////////////////////////////////////////////////

extern crate base;
use base::error;
