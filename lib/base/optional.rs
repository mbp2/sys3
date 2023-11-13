/// FFI-safe variant of [`Option`].
///
/// Implements the [`From`] and [`Into`] traits for easy conversion to and from [`Option`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum Optional<T> {
   /// Some value `T`.
   Some(T),

   /// No value.
   None,
}

impl<T> Optional<T> {
   pub fn into_option(&self) -> Option<T> {
      self.into()
   }

   pub const fn as_ref(&self) -> Option<&T> {
      match self {
         Self::Some(x) => Option::Some(x),
         Self::None => Option::None,
      }
   }

   /// Converts from `&mut Optional<T>` to `Option<&mut T>`.
   ///
   /// For convenience, this method directly performs the conversion to the standard
   /// [`Option`] type.
   pub fn as_mut(&mut self) -> Option<&mut T> {
      match self {
         Self::Some(x) => Option::Some(x),
         Self::None => Option::None,
      }
   }
}

impl<T> From<Option<T>> for Optional<T> {
   fn from(v: Option<T>) -> Optional<T> {
      match v {
         Some(v) => Optional::Some(v),
         None => Optional::None,
      }
   }
}

impl<T> From<Optional<T>> for Option<T> {
   fn from(value: Optional<T>) -> Self {
      match value {
         Optional::Some(value) => Option::Some(value),
         Optional::None => Option::None,
      }
   }
}
