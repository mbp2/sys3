pub type Result = core::result::Result<(), Box<dyn Error>>;

pub trait Error {
   fn msg() -> String;
}

// IMPORTS //

#[cfg(not(feature = "allocators"))]
use std_alloc::{boxed::Box, string::String};

#[cfg(feature = "allocators")]
use crate::{pointer::Unique as Box, string::String};
