pub const JSON_WHITESPACE: [u8; 4] = [b' ', b'\t', b'\n', b'\r'];
pub const JSON_SYNTAX: [u8; 6] = [
   JSON_COMMA, JSON_COLON, JSON_LEFT_BRACKET,
   JSON_RIGHT_BRACKET, JSON_LEFT_BRACE, JSON_RIGHT_BRACE
];

pub const JSON_COMMA: u8 = b',';
pub const JSON_COLON: u8 = b':';
pub const JSON_LEFT_BRACKET: u8 = b'[';
pub const JSON_RIGHT_BRACKET: u8 = b']';
pub const JSON_LEFT_BRACE: u8 = b'{';
pub const JSON_RIGHT_BRACE: u8 = b'}';
pub const JSON_QUOTE: u8 = b'"';

pub const FALSE_LEN: usize = "false".len();
pub const TRUE_LEN: usize = "true".len();
pub const NULL_LEN: usize = "null".len();

pub fn lex(mut string: String) -> Result<Array<impl JsonValue>, LexError> {
   let mut tokens: Array<impl JsonValue> = Array::new();

   for _ in string.bytes() {
      if let Ok(s) = lex_string(string.clone()) {
         tokens.push(s);
         continue;
      }

      if let Ok(number) = lex_number(string.clone()) {
         tokens.push(number);
         continue;
      }

      if let Ok(boolean) = lex_boolean(string.clone()) {
         tokens.push(boolean);
         continue;
      }

      if let Ok(null) = lex_null(string.clone()) {
         tokens.push(null);
         continue;
      }

      let x: &u8 = string.get(0).expect("expected char at 0 index");

      for y in JSON_WHITESPACE {
         if x == y {
            // Ignore whitespace.
            string = String::from(string.as_bytes()[1..]);
         } else {
            for z in JSON_SYNTAX {
               if x == z {
                  tokens.push(x);
                  string = String::from(string.as_bytes()[1..]);
               } else {
                  return Err(LexError::UnexpectedChar);
               }
            }
         }
      }
   }

   return Ok(tokens);
}

/// Lex the provided [`String`](alloc::string::String) and return the result.
pub fn lex_string(mut string: String) -> Result<String, LexError> {
   let mut json_string: String = String::new();

   if string.get(0) == JSON_QUOTE {
      let (_, slice) = string.split_at(1);
      string = String::from(slice);
   } else {
      return Ok(string);
   }

   for byte in string.bytes() {
      if byte == JSON_QUOTE {
         return Ok(json_string);
      } else {
         json_string.push(char::from(byte));
      }
   }

   return Err(LexError::ExpectedQuote);
}

pub fn lex_number(mut string: String) -> Result<impl JsonValue, LexError> {
   let mut json_float: f32 = 0.0;
   let mut json_number: i32 = 0;
   let mut json_string: String = String::new();
   let number_chars: [u8; 13] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'-', b'e', b'.'];

   for byte in string.bytes() {
      for c in number_chars {
         if c == byte {
            json_string.push(char::from(byte));
         } else {
            break;
         }
      }
   }

   return if json_string.contains('.') {
      json_float = json_string.parse().expect("expected a valid f32");
      Ok(json_float)
   } else {
      json_number = json_string.parse().expect("expected a valid i32");
      Ok(json_number)
   };
}

pub fn lex_boolean(mut string: String) -> Result<bool, LexError> {
   let strlen: usize = string.len();

   if strlen >= TRUE_LEN && string[..TRUE_LEN] == "true" {
      return Ok(true);
   } else if strlen >= FALSE_LEN && string[..FALSE_LEN] == "false" {
      return Ok(false);
   }

   return Err(LexError::UnexpectedChar);
}

pub fn lex_null(mut string: String) -> Result<bool, LexError> {
   let strlen: usize = string.len();

   if strlen >= NULL_LEN && string[..NULL_LEN] == "null" {
      return Ok(true);
   }

   return Err(LexError::UnexpectedChar);
}

pub trait Number{}

impl Number for i32{}
impl Number for f32{}

impl JsonValue for String{}
impl JsonValue for i32{}
impl JsonValue for f32{}
impl JsonValue for bool{}

// IMPORTS //

use {
   super::{data::JsonValue, error::LexError},
   crate::{array::Array, pointer::Unique},
   std_alloc::string::{String, ToString},
};
