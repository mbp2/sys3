/// The default global task executor.
pub static DEFAULT_EXECUTOR: Spinlock<Executor> = Spinlock::new(Executor::new());

/// Our executor type.
pub struct Executor {
   /// A collection of tasks to execute.
   pub tasks: TaskList,
}

impl Executor {
   pub const fn new() -> Self {
      return Executor{
         tasks: VecDeque::new(),
      };
   }

   pub fn add_task<T>(&mut self, task: Arc<Task<T>>)
   where
      T: Send + 'static, {
      self.tasks.push_back(Box::new(task));
   }

   /// Add a [`Task<T>`](crate::task::Task) to the executor queue and immediately poll it.
   pub fn poll_now<T>(&mut self, future: Pin<Box<dyn Future<Output = T> + 'static + Send>>)
   where
      T: Send + 'static {
      let task = Arc::new(Task{
         future: Spinlock::new(future),
         completed: AtomicBool::new(false),
      });

      task.update();
      self.add_task(task);
   }

   /// Add task for a future to the executor queue.
   pub fn add_async<T>(&mut self, future: Pin<Box<dyn Future<Output = T> + 'static + Send>>)
   where
      T: Send + 'static {
      let task = Arc::new(Task{
         future: Spinlock::new(future),
         completed: AtomicBool::new(false),
      });

      self.add_task(task);
   }

   /// Polls all pending tasks on global executor and remove completed tasks.
   ///
   /// You may notice, that when all tasks will done, we keep them, although this is objectively
   /// useless. I think, finding our each completed task from [`Wake::wake_by_ref()`] at [`Executor::tasks`]
   /// and drop it is too expensive, so we just mark them via [`Task<T>::completed`] field as done.
   /// When all tasks will done and we add new task and run them, old completed tasks will be removed
   /// from [`Executor::tasks`] or [`Executor::collect()`].
   pub fn run(&mut self) {
      for _ in 0..self.tasks.len() {
         let task = self.tasks.pop_front().unwrap();
         if !task.is_done() {
            task.update();
            self.tasks.push_back(task);
         }
      }
   }

   /// Removes completed task from [`Executor::tasks`].
   ///
   /// As you may also notice, same as [`Executor::run()`], but don't poll tasks.
   /// Only drop completed tasks.
   pub fn collect(&mut self) {
      for _ in 0..self.tasks.len() {
         let task = self.tasks.pop_front().unwrap();
         if !task.is_done() {
            self.tasks.push_back(task);
         }
      }
   }
}

// IMPORTS //

use {
   super::{Pendable, Task, TaskList},
   core::{
      future::Future,
      pin::Pin,
      sync::atomic::AtomicBool,
   },
   spinning_top::Spinlock,
   std_alloc::{
      boxed::Box,
      collections::VecDeque,
      sync::Arc,
   },
};
