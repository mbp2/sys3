pub type Result = core::result::Result<(), Box<dyn BaseError>>;

pub trait BaseError: Debug + Display {
   /// The lower-level source of error, if any.
   ///
   /// TODO: provide examples.
   fn source(&self) -> Option<&(dyn BaseError + 'static)> {
      return None;
   }

   /// Gets the [`TypeId`][core::any::TypeId] of `self`.
   fn type_id(&self, _: private::Internal) -> TypeId
   where
      Self: 'static, {
      return TypeId::of::<Self>();
   }

   /// Returns a stack backtrace.
   ///
   /// TODO: provide examples.
   fn backtrace(&self) -> Option<()> {
      return None;
   }
}

impl<'a, E: BaseError + 'a> From<E> for Box<dyn BaseError + 'a> {
   /// Converts a type of [`BaseError`] + [`Send`] + [`Sync`] into a unique pointer of
   /// dyn [`BaseError`] + [`Send`] + [`Sync`].
   fn from(value: E) -> Self {
      return Box::new(value);
   }
}

impl From<String> for Box<dyn BaseError + Send + Sync> {
   /// Converts a [`String`] into a unique pointer of
   /// dyn [`BaseError`] + [`Send`] + [`Sync`].
   fn from(value: String) -> Self {
      struct StringError(String);

      impl BaseError for StringError {}

      impl Display for StringError {
         fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            Display::fmt(&self.0, f)
         }
      }

      impl Debug for StringError {
         fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            Debug::fmt(&self.0, f)
         }
      }

      return Box::new(StringError(value));
   }
}

impl From<String> for Box<dyn BaseError> {
   /// Converts a [`String`] into a unique pointer of
   /// dyn [`BaseError`].
   fn from(value: String) -> Self {
      let e: Box<dyn BaseError + Send + Sync> = From::from(value);
      let h: Box<dyn BaseError> = e;

      return h;
   }
}

impl<'a> From<&str> for Box<dyn BaseError + Send + Sync + 'a> {
   /// Converts a [`str`][prim@str] into a unique pointer of dyn [`BaseError`] + [`Send`] + [`Sync`].
   #[inline]
   fn from(value: &str) -> Self {
      return From::from(String::from(value));
   }
}

impl From<&str> for Box<dyn BaseError> {
   /// Converts a [`str`][prim@str] into a unique pointer of dyn [`BaseError`].
   fn from(value: &str) -> Self {
      return From::from(String::from(value));
   }
}

impl BaseError for AllocationError{}
impl BaseError for LayoutError{}

// MODULES //

mod private {
   /// A hack to prevent `type_id` from being overridden by `BaseError`
   /// implementations, since that may enable unsound downcasting.
   #[derive(Debug)]
   pub struct Internal;
}

// IMPORTS //

#[cfg(not(feature="allocators"))]
use std_alloc::{boxed::Box, string::String};

#[cfg(feature="allocators")]
use crate::{pointer::Unique as Box, string::String};

use {
   crate::alloc::{AllocationError, layout::LayoutError},
   core::{
      any::TypeId,
      fmt::{self, Debug, Display, Formatter},
   },
};
