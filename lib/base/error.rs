pub type Result = core::result::Result<(), Box<dyn Error>>;

pub trait Error {
   fn msg() -> String;
}

// IMPORTS //

use std_alloc::{
   boxed::Box,
   string::String
};
