/// Polls all pending tasks on global executor and remove completed tasks.
pub fn run_tasks() {
   executor::DEFAULT_EXECUTOR.lock().run();
}

/// Adds task for a future to the global executor queue.
pub fn add_future<T>(future: impl Future<Output = T> + 'static + Send)
where
   T: Send + 'static, {
   executor::DEFAULT_EXECUTOR.lock().add_async(Box::pin(future));
}

/// Drops completed tasks and checks if any uncompleted tasks remain.
pub fn completed() -> bool {
   let mut executor = executor::DEFAULT_EXECUTOR.lock();
   executor.collect();
   return executor.tasks.is_empty();
}

/// Adds task for a future to the executor queue and immediately polls it.
pub fn poll_now<T>(future: impl Future<Output = T> + 'static + Send)
where
   T: Send + 'static, {
   executor::DEFAULT_EXECUTOR.lock().poll_now(Box::pin(future));
}

pub type TaskList = VecDeque<Box<dyn Pendable + core::marker::Send + core::marker::Sync>>;

/// Container for [`Future`] and [`Future`]'s state, like [`Task::completed`].
///
/// Task is our unit of execution and holds a future to wait on.
pub struct Task<T> {
   pub future: Spinlock<Pin<Box<dyn Future<Output = T> + Send + 'static>>>,
   pub completed: AtomicBool,
}

pub trait Pendable {
   /// Updates future progress via calling [`Future::poll()`].
   ///
   /// Shall contain future's status internally and corresponding to [`Future::poll`] state.
   fn update(&self);

   /// Returns `true` if the state is corresponding to [`core::task::Poll::Ready`] otherwise - false.
   ///
   /// Needed to determine, which task we shall drop.
   fn is_done(&self) -> bool;
}

impl<T> Wake for Task<T> {
   fn wake(self: Arc<Self>) {
      self.update();
   }

   fn wake_by_ref(self: &Arc<Self>) {
      self.update();
   }
}

impl<T> ArcWake for Task<T> {
   fn wake(self: Arc<Self>) {
      self.update();
   }

   fn wake_by_ref(arc_self: &Arc<Self>) {
      arc_self.update();
   }
}

impl<T> Pendable for Arc<Task<T>> {
   fn update(&self) {
      let mut future = self.future.lock();
      let waker = waker_ref(self);
      let context = &mut Context::from_waker(&waker);
      self.completed.store(
         !matches!(future.as_mut().poll(context), Poll::Pending),
         Ordering::Relaxed,
      );
   }

   fn is_done(&self) -> bool {
      return self.completed.load(Ordering::Relaxed);
   }
}

// MODULES //

pub mod executor;
pub mod keyboard;

// IMPORTS //

use {
   core::{
      future::Future,
      pin::Pin,
      sync::atomic::{AtomicBool, Ordering},
      task::{Context, Poll},
   },
   futures_util::task::{ArcWake, waker_ref},
   spinning_top::Spinlock,
   std_alloc::{
      boxed::Box,
      collections::VecDeque,
      sync::Arc,
      task::Wake
   },
};

// EXPORTS //

pub use self::executor::Executor;
