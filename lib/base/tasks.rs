pub struct Task {
   pub id: TaskId,
   pub future: Pin<Unique<dyn Future<Output = ()>>>,
}

impl Task {
   pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
      return Task{
         id: TaskId::new(),
         future: Unique::pin(future),
      };
   }

   pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
      return self.future.as_mut().poll(context);
   }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct TaskId(pub u64);

impl TaskId {
   pub fn new() -> Self {
      static NEXT_ID: AtomicU64 = AtomicU64::new(0);
      TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
   }
}

// MODULES //

pub mod executor;
pub mod keyboard;

// IMPORTS //

use {
   crate::pointer::Unique,
   core::{
      future::Future,
      pin::Pin,
      sync::atomic::{AtomicU64, Ordering},
      task::{Context, Poll},
   },
};

// EXPORTS //
