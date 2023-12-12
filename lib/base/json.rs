pub fn from_string(string: String) -> Option<Json> {
   return None;
}

// MODULES //

/// JSON data structures represented in the runtime.
pub mod data;

/// JSON parsing and lexing error handling.
pub mod error;

/// The lexer subsystem for our JSON runtime module.
pub mod lexer;

// IMPORTS //

use {
   self::data::Json,
   std_alloc::string::String,
};
