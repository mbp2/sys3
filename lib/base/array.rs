//! A dynamically sized array type.

pub struct Array<T, A: Allocator = GlobalAllocator> {
   size: usize,
   buf: RawArray<T, A>,
}

impl<T, A: Allocator> Array<T, A> {
   pub fn new_with(alloc: A) -> Self {
      Array {
         size: 0,
         buf: RawArray::new(alloc),
      }
   }

   pub fn resize_with<F>(&mut self, new_size: usize, f: F)
   where
      F: Fn() -> T, {
      if new_size < self.size && needs_drop::<T>() {
         for i in new_size..self.size {
            unsafe {
               drop_in_place(self.buf.pointer.offset(i as isize));
            }
         }
      } else if new_size > self.size {
         if new_size > self.buf.capacity {
            self.reserve(new_size);
         }

         for i in self.size..new_size {
            unsafe {
               self.buf.pointer.offset(i as isize).write(f());
            }
         }
      }

      self.size = new_size;
   }

   pub fn resize(&mut self, new_size: usize, value: T)
   where
      T: Clone, {
      self.resize_with(new_size, || value.clone());
   }

   pub fn resize_default(&mut self, new_size: usize)
   where
      T: Default, {
      self.resize_with(new_size, || T::default());
   }

   pub fn reserve(&mut self, new_capacity: usize) {
      self.buf.reserve(new_capacity);
   }

   fn grow_auto(&mut self) {
      let single_layout = Layout::new::<T>();

      let old_capacity_bytes = self.buf.capacity * single_layout.size;
      assert!(old_capacity_bytes <= (usize::MAX / 4));

      let new_capacity = if self.buf.capacity == 0 {
         1
      } else {
         self.buf.capacity * 2
      };

      self.reserve(new_capacity);
   }

   #[inline]
   pub fn len(&self) -> usize {
      self.size
   }

   #[inline]
   pub fn capacity(&self) -> usize {
      self.buf.capacity
   }

   pub fn push(&mut self, value: T) {
      if self.size == self.buf.capacity {
         self.grow_auto();
      }

      unsafe {
         self.buf.pointer.offset(self.size as isize).write(value);
      }

      self.size += 1;
   }

   pub fn pop(&mut self) -> Option<T> {
      if self.size == 0 {
         None
      } else {
         let value = unsafe { self.buf.pointer.offset((self.size - 1) as isize).read() };

         self.size -= 1;
         Some(value)
      }
   }

   pub fn clear(&mut self) {
      if needs_drop::<T>() {
         unsafe {
            for i in 0..self.size {
               drop_in_place(self.buf.pointer.offset(i as isize));
            }
         }
      }

      self.size = 0;
   }

   #[inline]
   pub fn is_empty(&self) -> bool {
      self.size == 0
   }
}

impl<T> Array<T, GlobalAllocator> {
   pub fn new() -> Self {
      Self::new_with(GlobalAllocator)
   }
}

impl<T, A: Allocator> Drop for Array<T, A> {
   fn drop(&mut self) {
      if !self.buf.pointer.is_null() {
         self.clear();
      }
   }
}

impl<T, A: Allocator> Deref for Array<T, A> {
   type Target = [T];

   #[inline]
   fn deref(&self) -> &Self::Target {
      unsafe { slice::from_raw_parts(self.buf.pointer, self.size) }
   }
}

impl<T, A: Allocator> DerefMut for Array<T, A> {
   fn deref_mut(&mut self) -> &mut Self::Target {
      unsafe { slice::from_raw_parts_mut(self.buf.pointer, self.size) }
   }
}

impl<T, A: Allocator> Extend<T> for Array<T, A> {
   fn extend<I>(&mut self, iter: I)
   where
      I: IntoIterator<Item = T>, {
      for e in iter {
         self.push(e);
      }
   }
}

impl<'a, T: 'a, A: Allocator> Extend<&'a T> for Array<T, A>
where
   T: Clone,
{
   fn extend<I>(&mut self, iter: I)
   where
      I: IntoIterator<Item = &'a T>, {
      for e in iter {
         self.push(e.clone());
      }
   }
}

impl<T, A: Allocator> FromIterator<T> for Array<T, A>
where
   A: Default,
{
   fn from_iter<I>(iter: I) -> Self
   where
      I: IntoIterator<Item = T>, {
      let mut array = Array::new_with(A::default());
      array.extend(iter);
      return array;
   }
}

pub struct IntoIter<T, A: Allocator> {
   inner: Array<T, A>,
   current: usize,
   size: usize,
}

impl<T, A: Allocator> Iterator for IntoIter<T, A> {
   type Item = T;

   fn next(&mut self) -> Option<T> {
      if self.current >= self.size {
         None
      } else {
         unsafe {
            let index = self.current;
            self.current += 1;
            Some(read(self.inner.buf.pointer.offset(index as isize)))
         }
      }
   }

   fn size_hint(&self) -> (usize, Option<usize>) {
      let remaining = self.size - self.current;
      (remaining, Some(remaining))
   }
}

impl<T, A: Allocator> Drop for IntoIter<T, A> {
   fn drop(&mut self) {
      // Drop the remaining elements if we didn't iter
      // until the end.
      if needs_drop::<T>() {
         unsafe {
            for i in self.current..self.size {
               drop_in_place(self.inner.buf.pointer.offset(i as isize))
            }
         }
      }

      // Set size of the array to 0 so it doesn't drop anything else.
      self.inner.size = 0;
   }
}

pub struct RawArray<T, A: Allocator> {
   pub pointer: *mut T,
   pub capacity: usize,
   pub allocator: A,
   _phantom: PhantomData<T>,
}

impl<T, A: Allocator> RawArray<T, A> {
   pub fn new(allocator: A) -> Self {
      let capacity = if size_of::<T>() == 0 { !0 } else { 0 };

      Self {
         pointer: ptr::null_mut(),
         capacity,
         allocator,
         _phantom: PhantomData,
      }
   }

   pub fn reserve(&mut self, new_capacity: usize) {
      if new_capacity <= self.capacity {
         return;
      }

      let pointer = unsafe {
         alloc_array::<T>(&mut self.allocator, new_capacity)
            .expect("Allocation error")
            .as_ptr()
      };

      if self.capacity > 0 {
         unsafe {
            pointer.copy_from(self.pointer as *mut u8, self.capacity);
            self
               .allocator
               .deallocate_aligned(self.pointer as *mut u8, Layout::from_size(self.capacity));
         }
      }

      self.pointer = pointer as *mut T;
      self.capacity = new_capacity;
   }
}

impl<T, A: Allocator> Drop for RawArray<T, A> {
   fn drop(&mut self) {
      if !self.pointer.is_null() {
         unsafe {
            self
               .allocator
               .deallocate_aligned(self.pointer as *mut u8, Layout::from_size(self.capacity));
         }
      }
   }
}

pub trait StackArray {
   type Element;

   fn len(&self) -> usize;
   fn as_ptr(&self) -> *const Self::Element;
   fn as_mut_ptr(&mut self) -> *mut Self::Element;
}

enum SmallArrayData<S, A = GlobalAllocator>
where
   S: StackArray,
   A: Allocator, {
   Stack(usize, ManuallyDrop<S>),
   Heap(Array<S::Element, A>),
}

pub struct SmallArray<S, A = GlobalAllocator>
where
   S: StackArray,
   A: Allocator, {
   alloc: Option<A>,
   data: SmallArrayData<S, A>,
}

impl<S, A> SmallArray<S, A>
where
   S: StackArray,
   A: Allocator,
{
   #[inline]
   pub fn len(&self) -> usize {
      self.get_infos().1
   }

   #[inline]
   pub fn capacity(&self) -> usize {
      self.get_infos().2
   }

   pub fn reserve(&mut self, new_cap: usize) {
      if new_cap <= self.capacity() {
         return;
      }

      if let SmallArrayData::Stack(used, array) = &mut self.data {
         if new_cap > array.len() {
            let alloc = self.alloc.take().unwrap();
            let mut new_array = Array::new_with(alloc);
            new_array.reserve(new_cap);

            let ptr = array.as_mut_ptr();
            for i in 0..*used {
               new_array.push(unsafe { ptr::read(ptr.offset(i as isize)) });
            }

            self.data = SmallArrayData::Heap(new_array);
         }
      }

      if let SmallArrayData::Heap(array) = &mut self.data {
         array.reserve(new_cap);
      }
   }

   pub fn new_with(alloc: A) -> Self {
      Self {
         alloc: Some(alloc),
         data: SmallArrayData::Stack(0, unsafe { MaybeUninit::uninit().assume_init() }),
      }
   }

   #[inline]
   fn get_infos(&self) -> (*const S::Element, usize, usize) {
      match &self.data {
         SmallArrayData::Stack(used, array) => (array.as_ptr(), *used, array.len()),
         SmallArrayData::Heap(array) => (array.as_ptr(), array.len(), array.capacity()),
      }
   }

   #[inline]
   fn get_infos_mut(&mut self) -> (*mut S::Element, usize, usize) {
      match &mut self.data {
         SmallArrayData::Stack(used, array) => (array.as_mut_ptr(), *used, array.len()),
         SmallArrayData::Heap(array) => (array.as_mut_ptr(), array.len(), array.capacity()),
      }
   }

   pub fn push(&mut self, element: S::Element) {
      let (_, len, cap) = self.get_infos_mut();
      if len == cap {
         self.reserve(len * 2);
      }

      match &mut self.data {
         SmallArrayData::Stack(used, array) => {
            unsafe {
               ptr::write(array.as_mut_ptr().offset(*used as isize), element);
            }
            *used += 1;
         }
         SmallArrayData::Heap(array) => {
            array.push(element);
         }
      }
   }

   pub fn pop(&mut self) -> Option<S::Element> {
      if self.len() == 0 {
         return None;
      }

      match &mut self.data {
         SmallArrayData::Stack(used, array) => {
            *used -= 1;
            unsafe { Some(ptr::read(array.as_mut_ptr().offset(*used as isize))) }
         }
         SmallArrayData::Heap(array) => array.pop(),
      }
   }

   pub fn clear(&mut self) {
      match &mut self.data {
         SmallArrayData::Stack(used, array) => {
            if needs_drop::<S::Element>() {
               for i in 0..*used {
                  unsafe {
                     ptr::drop_in_place(array.as_mut_ptr().offset(i as isize));
                  }
               }
            }
            *used = 0;
         }
         SmallArrayData::Heap(array) => array.clear(),
      }
   }

   #[inline]
   pub fn is_empty(&self) -> bool {
      self.len() == 0
   }

   pub fn resize_with<F>(&mut self, new_size: usize, f: F)
   where
      F: Fn() -> S::Element, {
      self.reserve(new_size);

      let (ptr, len, _) = self.get_infos_mut();

      match &mut self.data {
         SmallArrayData::Stack(used, _) => {
            if new_size < len && needs_drop::<S::Element>() {
               for i in new_size..len {
                  unsafe {
                     ptr::drop_in_place(ptr.offset(i as isize));
                  }
               }
            } else if new_size > len {
               for i in len..new_size {
                  unsafe {
                     ptr::write(ptr.offset(i as isize), f());
                  }
               }
            }

            *used = new_size;
         }
         SmallArrayData::Heap(array) => array.resize_with(new_size, f),
      }
   }

   pub fn resize(&mut self, new_size: usize, value: S::Element)
   where
      S::Element: Clone, {
      self.resize_with(new_size, || value.clone());
   }

   pub fn resize_default(&mut self, new_size: usize)
   where
      S::Element: Default, {
      self.resize_with(new_size, || S::Element::default());
   }
}

impl<S, A> Drop for SmallArray<S, A>
where
   S: StackArray,
   A: Allocator,
{
   fn drop(&mut self) {
      self.clear();
   }
}

impl<S, A> Deref for SmallArray<S, A>
where
   S: StackArray,
   A: Allocator,
{
   type Target = [S::Element];

   #[inline]
   fn deref(&self) -> &Self::Target {
      let (ptr, len, _) = self.get_infos();
      unsafe { slice::from_raw_parts(ptr, len) }
   }
}

impl<S, A> DerefMut for SmallArray<S, A>
where
   S: StackArray,
   A: Allocator,
{
   #[inline]
   fn deref_mut(&mut self) -> &mut Self::Target {
      let (ptr, len, _) = self.get_infos_mut();
      unsafe { slice::from_raw_parts_mut(ptr, len) }
   }
}

impl<S> SmallArray<S, GlobalAllocator>
where
   S: StackArray,
{
   pub fn new() -> Self {
      Self::new_with(GlobalAllocator)
   }
}

impl<T, S, A: Allocator> Extend<T> for SmallArray<S, A>
where
   T: Borrow<S::Element>,
   S: StackArray,
   S::Element: Clone,
{
   fn extend<I>(&mut self, iter: I)
   where
      I: IntoIterator<Item = T>, {
      for e in iter {
         self.push(e.borrow().clone());
      }
   }
}

macro impl_stack_array($len:expr, $name:ident) {
   impl<T> StackArray for [T; $len] {
      type Element = T;

      #[inline]
      fn len(&self) -> usize {
         $len
      }

      #[inline]
      fn as_ptr(&self) -> *const Self::Element {
         (self as &[Self::Element]).as_ptr()
      }

      #[inline]
      fn as_mut_ptr(&mut self) -> *mut Self::Element {
         (self as &mut [Self::Element]).as_mut_ptr()
      }
   }

   pub type $name<T, A = GlobalAllocator> = SmallArray<[T; $len], A>;
}

// @Todo Re-do this when const generics
impl_stack_array!(1, SmallArray1);
impl_stack_array!(2, SmallArray2);
impl_stack_array!(4, SmallArray4);
impl_stack_array!(8, SmallArray8);
impl_stack_array!(16, SmallArray16);
impl_stack_array!(24, SmallArray24);
impl_stack_array!(32, SmallArray32);
impl_stack_array!(64, SmallArray64);
impl_stack_array!(128, SmallArray128);

// IMPORTS //

use {
   crate::alloc::{alloc_array, Allocator, GlobalAllocator, Layout},
   core::{
      borrow::Borrow,
      iter::{FromIterator, IntoIterator},
      marker::PhantomData,
      mem::{needs_drop, size_of, ManuallyDrop, MaybeUninit},
      ops::{Deref, DerefMut},
      ptr::{self, drop_in_place, read},
      slice,
   },
};
