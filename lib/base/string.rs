/// Represents a growable UTF-8 encoded string.
pub struct String<A: Allocator = GlobalAllocator> {
   buffer: Array<u8, A>,
}

impl<A: Allocator> String<A> {
   /// Initialises an empty String with the specified allocator `A`
   ///
   /// ```
   /// use base::alloc::GlobalAllocator;
   /// use base::string::String;
   ///
   /// fn main() {
   ///   let s = String::new_with(GlobalAllocator);
   /// }
   /// ```
   pub fn new_with(alloc: A) -> Self {
      Self {
         buffer: Array::new_with(alloc),
      }
   }

   /// Initialises a `String` from a given `&str` using the specified allocator, `A`.
   ///
   ///
   /// ```
   /// use base::alloc::GlobalAllocator;
   /// use base::string::String;
   ///
   /// fn main()
   /// {
   ///   let s = String::from_str_with("hello!", GlobalAllocator);
   /// }
   /// ```
   pub fn from_str_with(s: &str, alloc: A) -> Self {
      let slice = s.as_bytes();
      let mut buf = Array::new_with(alloc);
      buf.resize(slice.len(), 0);

      unsafe {
         ptr::copy_nonoverlapping(s.as_ptr(), buf.as_mut_ptr(), slice.len());
      }

      return Self { buffer: buf };
   }

   /// Dereferences to the base `&str`.
   #[inline]
   pub fn as_str(&self) -> &str {
      self
   }

   /// Pushes a Unicode character point to the current instance of `String`.
   pub fn push(&mut self, c: char) {
      let mut bytes = [0u8; 4];
      c.encode_utf8(&mut bytes);
      self.buffer.extend(bytes[0..c.len_utf8()].iter());
   }
}

impl<A: Allocator> TryFrom<Array<u8, A>> for String<A> {
   type Error = core::str::Utf8Error;

   fn try_from(array: Array<u8, A>) -> Result<Self, Self::Error> {
      str::from_utf8(&array)?;
      Ok(Self { buffer: array })
   }
}

impl String<GlobalAllocator> {
   /// Initialises an empty `String`.
   ///
   ///
   /// ```
   /// use base::string::String;
   ///
   /// fn main()
   /// {
   ///   let s = String::new();
   /// }
   /// ```
   pub fn new() -> Self {
      Self::new_with(GlobalAllocator)
   }

   /// Initialises a `String` from a given `&str`.
   ///
   ///
   /// ```
   /// use base::string::String;
   ///
   /// fn main()
   /// {
   ///    let s = String::from("hello!");
   /// }
   /// ```
   pub fn from(s: &str) -> Self {
      Self::from_str_with(s, GlobalAllocator)
   }
}

impl<A: Allocator> AsRef<str> for String<A> {
   /// References the current `String` as a `&str`.
   #[inline]
   fn as_ref(&self) -> &str {
      self
   }
}

impl<A: Allocator> Borrow<str> for String<A> {
   /// Borrows the current `String` as `&str`.
   #[inline]
   fn borrow(&self) -> &str {
      self
   }
}

impl<A: Allocator> Deref for String<A> {
   type Target = str;

   /// Dereferences the current `String` into a `&str`.
   ///
   ///
   /// ```no_compile
   /// use base::string::String;
   ///
   /// fn main()
   /// {
   ///    let s = String::from("hello!");
   ///    let s = s.deref();
   /// }
   /// ```
   #[inline]
   fn deref(&self) -> &Self::Target {
      unsafe { str::from_utf8_unchecked(&self.buffer) }
   }
}

unsafe impl Send for String<GlobalAllocator> {}
unsafe impl Sync for String<GlobalAllocator> {}

impl<A: Allocator> DerefMut for String<A> {
   /// Dereferences the current `String` into a mutable `&str`.
   #[inline]
   fn deref_mut(&mut self) -> &mut str {
      unsafe { str::from_utf8_unchecked_mut(&mut self.buffer) }
   }
}

impl<A: Allocator> fmt::Display for String<A> {
   // allows println!() to work
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      fmt::Display::fmt(self.as_str(), f)
   }
}

impl<A: Allocator> fmt::Debug for String<A> {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      fmt::Display::fmt(self.as_str(), f)
   }
}

impl<A, T> PartialEq<T> for String<A>
where
   A: Allocator,
   T: AsRef<str>,
{
   #[inline]
   fn eq(&self, other: &T) -> bool {
      PartialEq::eq(self.as_str(), other.as_ref())
   }
}

impl<A: Allocator> Eq for String<A> {}

impl<A: Allocator> Hash for String<A> {
   fn hash<H: Hasher>(&self, h: &mut H) {
      Hash::hash(self.as_str(), h);
   }
}

//------------------------------------------------------------
//StringWide: A growable UTF-16 string.

/// Represents a growable, UTF-16-encoded String.
pub struct StringWide<A: Allocator = GlobalAllocator> {
   buf: Array<u16, A>,
}

impl<A: Allocator> StringWide<A> {
   /// Initialises an empty `StringWide` with the specified allocator, `A`.
   ///
   ///
   /// ```no_run
   /// use base::alloc::GlobalAllocator;
   /// use base::string::StringWide;
   ///
   /// fn main() {
   ///   let s = StringWide::new();
   /// }
   /// ```
   pub fn new_with(alloc: A) -> Self {
      Self {
         buf: Array::new_with(alloc),
      }
   }

   /// Initialises a `StringWide` from the given `&str` using the specified allocator, `A`.
   ///
   ///
   /// ```no_run
   /// use base::alloc::GlobalAllocator;
   /// use base::string::StringWide;
   ///
   /// fn main()
   /// {
   ///   let s = StringWide::from_str_with("hello!", GlobalAllocator);
   /// }
   /// ```
   pub fn from_str_with(s: &str, alloc: A) -> Self {
      let w_iter = s.encode_utf16();

      let mut buf = Array::new_with(alloc);
      buf.reserve(w_iter.size_hint().0);

      for wchar in w_iter {
         buf.push(wchar);
      }

      Self { buf }
   }

   /// See `String::push`.
   #[inline]
   pub fn push(&mut self, c: char) {
      let len = c.len_utf16();
      self.buf.resize(self.buf.len() + len, 0);

      let start = self.buf.len() - len;
      c.encode_utf16(&mut self.buf[start..]);
   }
}

impl StringWide<GlobalAllocator> {
   /// Initialises an empty `StringWide`.
   ///
   ///
   /// ```
   /// use base::string::StringWide;
   ///
   /// fn main() {
   ///   let s = StringWide::new();
   /// }
   /// ```
   pub fn new() -> Self {
      Self::new_with(GlobalAllocator)
   }

   /// Initialises a `StringWide` from the given `&str`.
   ///
   ///
   /// ```
   /// use base::string::StringWide;
   ///
   /// fn main()
   /// {
   ///   let s = StringWide::from("hello!");
   /// }
   /// ```
   pub fn from(s: &str) -> Self {
      Self::from_str_with(s, GlobalAllocator)
   }
}

impl<A: Allocator> AsRef<[u16]> for StringWide<A> {
   #[inline]
   fn as_ref(&self) -> &[u16] {
      &self.buf
   }
}

impl<A: Allocator> Deref for StringWide<A> {
   type Target = [u16];

   #[inline]
   fn deref(&self) -> &[u16] {
      &self.buf
   }
}

impl<A: Allocator> DerefMut for StringWide<A> {
   #[inline]
   fn deref_mut(&mut self) -> &mut [u16] {
      &mut self.buf
   }
}

unsafe impl Send for StringWide<GlobalAllocator> {}
unsafe impl Sync for StringWide<GlobalAllocator> {}

// IMPORTS //

use {
   crate::{
      alloc::{Allocator, GlobalAllocator},
      array::Array,
   },
   core::{
      borrow::Borrow,
      cmp::{Eq, PartialEq},
      fmt,
      hash::{Hash, Hasher},
      ops::{Deref, DerefMut},
      ptr, str,
   },
};
