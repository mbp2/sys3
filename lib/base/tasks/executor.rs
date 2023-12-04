pub struct Executor {
   pub tasks: BTreeMap<TaskId, Task>,
   pub task_queue: Arc<ArrayQueue<TaskId>>,
   pub waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
   pub fn new() -> Self {
      return Executor{
         tasks: BTreeMap::new(),
         task_queue: Arc::new(ArrayQueue::new(100)),
         waker_cache: BTreeMap::new(),
      };
   }

   pub fn spawn(&mut self, task: Task) {
      let task_id = task.id;
      if self.tasks.insert(task.id, task).is_some() {
         log::error!("A task with the same identifier already exists");
      }

      self.task_queue.push(task_id).expect("queue full");
   }

   pub fn run(&mut self) -> ! {
      loop{
         self.run_ready_tasks();
         self.sleep_if_idle();
      }
   }

   pub fn run_ready_tasks(&mut self) {
      // Destructure self to avoid borrow checker errors.
      let Self {
         tasks,
         task_queue,
         waker_cache,
      } = self;

      while let Some(task_id) = task_queue.pop() {
         let task = match tasks.get_mut(&task_id) {
            Some(task) => task,
            None => continue,
         };

         let waker = waker_cache.entry(task_id)
            .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));

         let mut context = Context::from_waker(waker);
      }
   }

   pub fn sleep_if_idle(&self) {
      use x86_64::instructions::interrupts::{self, enable_and_hlt};

      interrupts::disable();
      if self.task_queue.is_empty() {
         enable_and_hlt();
      } else {
         interrupts::enable();
      }
   }
}

pub struct TaskWaker {
   pub task_id: TaskId,
   pub task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
   pub fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
      Waker::from(Arc::new(TaskWaker {
         task_id,
         task_queue,
      }))
   }

   pub fn wake_task(&self) {
      self.task_queue.push(self.task_id).expect("task queue full");
   }
}

impl Wake for TaskWaker {
   fn wake(self: Arc<Self>) {
      self.wake_task();
   }

   fn wake_by_ref(self: &Arc<Self>) {
      self.wake_task();
   }
}

// IMPORTS //

use {
   super::{Task, TaskId},
   std_alloc::{collections::BTreeMap, sync::Arc, task::Wake},
   core::task::{Context, Poll, Waker},
   crossbeam_queue::ArrayQueue,
};
