/// An intrusive linked list
///
/// A clean room implementation of the one used in CS140e 2018 Winter
///
/// Thanks Sergio Benitez for his excellent work,
/// See [CS140e](https://cs140e.sergio.bz/) for more information
#[derive(Clone, Copy)]
pub struct LinkedList {
   pub head: *mut usize,
}

unsafe impl Send for LinkedList{}

impl LinkedList {
   /// Creates a new [`LinkedList`](LinkedList).
   pub const fn new() -> LinkedList {
      return LinkedList{
         head: ptr::null_mut(),
      };
   }

   /// Return `true` if the list is empty.
   pub fn empty(&self) -> bool {
      return self.head.is_null();
   }

   /// Push `item` to the front of the list.
   ///
   /// ## Arguments
   ///
   /// * `item` - A [`*mut usize`](prim@usize) item list.
   ///
   /// ## Safety
   ///
   /// Unsafe because we are working with raw pointers.
   pub unsafe fn push(&mut self, item: *mut usize) {
      *item = self.head as usize;
      self.head = item;
   }

   /// Try to remove the first item from the list.
   pub fn pop(&mut self) -> Option<*mut usize> {
      return match self.empty() {
         true => None,
         false => {
            let item: *mut usize = self.head;
            self.head = unsafe{ *item as *mut usize };
            Some(item)
         }
      };
   }

   /// Get an iterator over the items in the list.
   pub fn iterator(&self) -> Iter {
      return Iter{
         current: self.head,
         list: PhantomData,
      }
   }

   /// Get a mutable iterator over the items in the list.
   pub fn iterator_mut(&mut self) -> IterMut {
      return IterMut{
         previous: &mut self.head as *mut *mut usize as *mut usize,
         current: self.head,
         list: PhantomData,
      };
   }
}

impl Debug for LinkedList {
   fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
      f.debug_list().entries(self.iterator()).finish()
   }
}

/// A mutable list node in [`LinkedList`](crate::alloc::list::LinkedList)
pub struct ListNode {
   pub previous: *mut usize,
   pub current: *mut usize,
}

impl ListNode {
   pub fn pop(self) -> *mut usize {
      // Skip the current item.
      unsafe{
         *(self.previous) = *(self.current);
      }

      return self.current;
   }

   pub fn value(&self) -> *mut usize {
      return self.current;
   }
}

pub struct Iter<'a> {
   pub current: *mut usize,
   list: PhantomData<&'a LinkedList>,
}

impl<'a> Iterator for Iter<'a> {
   type Item = *mut usize;

   fn next(&mut self) -> Option<Self::Item> {
      return if self.current.is_null() {
         None
      } else {
         let item: *mut usize = self.current;
         let next: *mut usize = unsafe{ *item as *mut usize };
         self.current = next;

         Some(item)
      }
   }
}

/// A mutable interior over the linked list.
pub struct IterMut<'a> {
   list: PhantomData<&'a mut LinkedList>,
   pub current: *mut usize,
   pub previous: *mut usize,
}

impl<'a> Iterator for IterMut<'a> {
   type Item = ListNode;

   fn next(&mut self) -> Option<Self::Item> {
      return if self.current.is_null() {
         None
      } else {
         let result = ListNode{
            current: self.current,
            previous: self.previous,
         };

         self.previous = self.current;
         self.current = unsafe{ *self.current as *mut usize };

         Some(result)
      };
   }
}

// IMPORTS //

use {
   core::{
      fmt::{Debug, Formatter},
      marker::PhantomData,
      ptr,
   },
};
