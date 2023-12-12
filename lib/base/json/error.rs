/// Error lexing JSON input.
#[derive(Debug)]
pub enum LexError {
   /// Raised when not finding a begin or end quote.
   ExpectedQuote,
   /// Raised upon finding an unexpected code point.
   UnexpectedChar,
}

impl Display for LexError {
   fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
      match self {
         Self::ExpectedQuote => writeln!(f, "Expected quote in JSON string"),
         Self::UnexpectedChar => writeln!(f, "Unexpected char in provided JSON string"),
      }
   }
}

impl BaseError for LexError{}

// IMPORTS //

use {
   crate::error::BaseError,
   core::fmt::{Display, Formatter},
};
