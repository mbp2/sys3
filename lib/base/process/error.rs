#[derive(Debug)]
pub enum ProcError {
   NoStart,
   BadPriority,
}

impl fmt::Display for ProcError {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
         Self::NoStart => writeln!(f, "could not start process"),
         Self::BadPriority => writeln!(f, "bad task priority value"),
      }
   }
}

impl BaseError for ProcError{}

// IMPORTS //

use {
   crate::error::BaseError,
   core::fmt,
};
