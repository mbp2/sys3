// This type provides a lock based on busy waiting to realize mutual exclusion of tasks.
///
/// # Description
///
/// This structure behaves a lot like a normal Mutex. There are some differences:
///
/// - Interrupts save lock => Interrupts will be disabled
/// - By using busy waiting, it can be used outside the runtime.
/// - It is a so called ticket lock (https://en.wikipedia.org/wiki/Ticket_lock)
///   and completly fair.
///
/// The interface is derived from https://mvdnes.github.io/rust-docs/spin-rs/spin/index.html.
///
/// # Simple examples
///
/// ```
/// let spinlock = base::sync::SpinlockIrqSave::new(0);
///
/// // Modify the data
/// {
///     let mut data = spinlock.lock();
///     *data = 2;
/// }
///
/// // Read the data
/// let answer =
/// {
///     let data = spinlock.lock();
///     *data
/// };
///
/// assert_eq!(answer, 2);
/// ```
pub struct SpinlockIrqSave<T: ?Sized> {
   queue: AtomicUsize,
   dequeue: AtomicUsize,
   irq: AtomicBool,
   data: UnsafeCell<T>,
}

/// A guard to which the protected data can be accessed
///
/// When the guard falls out of scope it will release the lock.
#[derive(Debug)]
pub struct SpinlockIrqSaveGuard<'a, T: ?Sized + 'a> {
   //queue: &'a AtomicUsize,
   dequeue: &'a AtomicUsize,
   irq: &'a AtomicBool,
   data: &'a mut T,
}

// Same unsafe impls as `std::sync::Mutex`
unsafe impl<T: ?Sized + Send> Sync for SpinlockIrqSave<T> {}
unsafe impl<T: ?Sized + Send> Send for SpinlockIrqSave<T> {}

impl<T> SpinlockIrqSave<T> {
   pub const fn new(user_data: T) -> SpinlockIrqSave<T> {
      return SpinlockIrqSave{
         queue: AtomicUsize::new(0),
         dequeue: AtomicUsize::new(1),
         irq: AtomicBool::new(false),
         data: UnsafeCell::new(user_data),
      };
   }

   /// Consumes this mutex, returning the underlying data.
   pub fn into_inner(self) -> T {
      // We know statically that there are no outstanding references to
      // `self` so there's no need to lock.
      let SpinlockIrqSave{ data, .. } = self;
      data.into_inner()
   }
}

impl<T: ?Sized> SpinlockIrqSave<T> {
   fn obtain_lock(&self) {
      let irq = arch::irq::irq_nested_disable();
      let ticket = self.queue.fetch_add(1, Ordering::SeqCst) + 1;

      while self.dequeue.load(Ordering::SeqCst) != ticket {
         arch::irq::irq_nested_enable(irq);
         syscall::pause();
         arch::irq::irq_nested_disable();
      }

      self.irq.store(irq, Ordering::SeqCst);
   }

   pub fn lock(&self) -> SpinlockIrqSaveGuard<T> {
      self.obtain_lock();
      return SpinlockIrqSaveGuard{
         //queue: &self.queue,
         dequeue: &self.dequeue,
         irq: &self.irq,
         data: unsafe { &mut *self.data.get() },
      };
   }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SpinlockIrqSave<T> {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "irq: {:?} ", self.irq)?;
      write!(f, "queue: {} ", self.queue.load(Ordering::SeqCst))?;
      write!(f, "dequeue: {} ", self.dequeue.load(Ordering::SeqCst))?;
      write!(f, "data: {:?}", self.data.get())
   }
}

impl<T: ?Sized + Default> Default for SpinlockIrqSave<T> {
   fn default() -> SpinlockIrqSave<T> {
      SpinlockIrqSave::new(Default::default())
   }
}

impl<'a, T: ?Sized> Deref for SpinlockIrqSaveGuard<'a, T> {
   type Target = T;
   fn deref<'b>(&'b self) -> &'b T {
      &*self.data
   }
}

impl<'a, T: ?Sized> DerefMut for SpinlockIrqSaveGuard<'a, T> {
   fn deref_mut<'b>(&'b mut self) -> &'b mut T {
      &mut *self.data
   }
}

impl<'a, T: ?Sized> Drop for SpinlockIrqSaveGuard<'a, T> {
   /// The dropping of the SpinlockGuard will release the lock it was created from.
   fn drop(&mut self) {
      let irq = self.irq.swap(false, Ordering::SeqCst);
      self.dequeue.fetch_add(1, Ordering::SeqCst);
      arch::irq::irq_nested_enable(irq);
   }
}

// IMPORTS //

use {
   crate::{arch, syscall},
   core::{
      cell::UnsafeCell,
      fmt,
      marker::Sync,
      ops::{Deref, DerefMut, Drop},
      sync::atomic::{AtomicBool, AtomicUsize, Ordering},
   },
};
