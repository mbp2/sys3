pub const REALTIME_PRIORITY: TaskPriority = TaskPriority::from(NUM_PRIORITIES as u8 - 1);
pub const HIGH_PRIORITY: TaskPriority = TaskPriority::from(24);
pub const NORMAL_PRIORITY: TaskPriority = TaskPriority::from(16);
pub const LOW_PRIORITY: TaskPriority = TaskPriority::from(0);

pub trait TaskFrame {
   /// Create the initial stack frame for a new task
   fn create_stack_frame(&mut self, func: extern "C" fn());
}

/// A task control block, which identifies either a process or a thread
#[repr(align(64))]
pub struct Task {
   /// Unique identifier of the task.
   pub id: TaskId,

   /// Priority of the task.
   pub priority: TaskPriority,

   /// Status of a task, e.g. if a task is ready or blocked.
   pub status: TaskStatus,

   /// Last stack pointer before a context switch to another task.
   pub last_stack_pointer: usize,

   /// The task stack.
   pub stack: Unique<dyn Stack>,

   /// Physical address of the first-level page table.
   pub root_page_table: usize,

   /// The next task in the queue.
   pub next: Option<Rc<RefCell<Task>>>,

   /// The previous task in the queue.
   pub prev: Option<Rc<RefCell<Task>>>,
}

impl Task {
   pub fn new_idle(id: TaskId) -> Task {
      return Task{
         id: id,
         priority: LOW_PRIORITY,
         status: TaskStatus::Idle,
         last_stack_pointer: 0,
         stack: Unique::new(crate::arch::x86_64::get_boot_stack()),
         root_page_table: crate::arch::x86_64::memory::get_kernel_root_page_table(),
         next: None,
         prev: None,
      };
   }

   pub fn new(id: TaskId, status: TaskStatus, priority: TaskPriority) -> Task {
      return Task{
         id: id,
         priority: priority,
         status: status,
         last_stack_pointer: 0,
         stack: Unique::new(TaskStack::new()),
         root_page_table: crate::arch::x86_64::memory::get_kernel_root_page_table(),
         next: None,
         prev: None,
      };
   }
}

impl Drop for Task {
   fn drop(&mut self) {
      if self.root_page_table != crate::arch::x86_64::memory::get_kernel_root_page_table() {
         log::debug!(
            "Deallocate page table `0x{:x} of task {}",
            self.root_page_table, self.id
         );

         // TODO: deallocate physical memory used for task.
      }
   }
}

#[repr(C)]
#[repr(align(64))]
#[derive(Clone, Copy)]
pub struct TaskStack {
   pub buffer: [u8; STACK_SIZE],
}

impl TaskStack {
   pub const fn new() -> TaskStack {
      return TaskStack{
         buffer: [0; STACK_SIZE],
      };
   }
}

impl Stack for TaskStack {
   fn top(&self) -> usize {
      return (&(self.buffer[STACK_SIZE - 16]) as *const _) as usize;
   }

   fn bottom(&self) -> usize {
      return (&(self.buffer[0]) as *const _) as usize;
   }
}

/// A unique task identifier, i.e. `pid`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct TaskId(pub u64);

impl TaskId {
   pub const fn into(self) -> u64 {
      return self.0;
   }

   pub const fn from(x: u64) -> TaskId {
      return TaskId(x);
   }
}

impl Display for TaskId {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "{}", self.0)
   }
}

/// Priority of a task
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct TaskPriority(pub u8);

impl TaskPriority {
   pub const fn into(self) -> u8 {
      return self.0;
   }

   pub const fn from(x: u8) -> Self {
      return TaskPriority(x);
   }
}

impl Display for TaskPriority {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "{}", self.0)
   }
}

pub struct QueueHead {
   pub head: Option<Rc<RefCell<Task>>>,
   pub tail: Option<Rc<RefCell<Task>>>,
}

impl Default for QueueHead {
   fn default() -> Self {
      return QueueHead{
         head: None,
         tail: None,
      };
   }
}

/// Realise a priority queue for tasks.
pub struct PriorityTaskQueue {
   pub queues: [QueueHead; NUM_PRIORITIES],
   pub prio_bitmap: u64,
}

impl PriorityTaskQueue {
   /// Creates an empty priority queue for tasks
   pub fn new() -> Self {
      return PriorityTaskQueue{
         queues: Default::default(),
         prio_bitmap: 0,
      };
   }

   /// Add a task by its priority to the queue
   pub fn push(&mut self, task: Rc<RefCell<Task>>) {
      let i = task.borrow().priority.into() as usize;

      self.prio_bitmap |= 1 << i;
      match self.queues[i].tail {
         None => {
            self.queues[i].head = Some(task.clone());

            let mut borrow = task.borrow_mut();
            borrow.next = None;
            borrow.prev = None;
         }

         Some(ref mut tail) => {
            // Add task at the end of the node
            tail.borrow_mut().next = Some(task.clone());

            let mut borrow = task.borrow_mut();
            borrow.next = None;
            borrow.prev = None;
         }
      }

      self.queues[i].tail = Some(task.clone());
   }

   pub fn pop_from_queue(&mut self, qi: usize) -> Option<Rc<RefCell<Task>>> {
      let new_head;
      let task;

      match self.queues[qi].head {
         None => return None,
         Some(ref mut head) => {
            let mut borrow = head.borrow_mut();
            match borrow.next {
               Some(ref mut nhead) => {
                  nhead.borrow_mut().prev = None;
               }
               None => {}
            }

            new_head = borrow.next.clone();
            borrow.next = None;
            borrow.prev = None;

            task = head.clone();
         }
      }

      self.queues[qi].head = new_head;
      if self.queues[qi].head.is_none() {
         self.queues[qi].tail = None;
         self.prio_bitmap &= !(1 << qi as u64);
      }

      return Some(task);
   }

   /// Pop the task with the highest priority from the queue
   pub fn pop(&mut self) -> Option<Rc<RefCell<Task>>> {
      if let Some(i) = call_msb(self.prio_bitmap) {
         return self.pop_from_queue(i as usize);
      }

      return None;
   }

   /// Pop the next task, which has a higher or the same priority as `priority`.
   pub fn pop_with_priority(&mut self, priority: TaskPriority) -> Option<Rc<RefCell<Task>>> {
      if let Some(i) = call_msb(self.prio_bitmap) {
         if i >= priority.into() as u64 {
            return self.pop_from_queue(i as usize);
         }
      }

      return None;
   }

   pub fn remove(&mut self, task: Rc<RefCell<Task>>) {
      let i = task.borrow().priority.into() as usize;

      let mut current = self.queues[i].head.clone();
      let mut next_current;

      loop{
         match current{
            None => break,
            Some(ref current_task) => {
               if Rc::ptr_eq(&current_task, &task) {
                  let (mut prev, mut next) = {
                     let borrowed = current_task.borrow_mut();
                     (borrowed.prev.clone(), borrowed.next.clone())
                  };

                  match prev {
                     Some(ref mut t) => {
                        t.borrow_mut().next = next.clone();
                     }

                     None => {}
                  };

                  match next {
                     Some(ref mut t) => {
                        t.borrow_mut().prev = prev.clone();
                     }

                     None => {}
                  };

                  break;
               }

               next_current = current_task.borrow().next.clone();
            }
         }

         current = next_current.clone();
      }

      let new_head = match self.queues[i].head {
         Some(ref current_task) => {
            if Rc::ptr_eq(&current_task, &task) {
               true
            } else {
               false
            }
         }

         None => false,
      };

      if new_head {
         self.queues[i].head = task.borrow().next.clone();
         if self.queues[i].head.is_none() {
            self.prio_bitmap &= !(1 << i as u64);
         }
      }
   }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskStatus {
   Blocked,
   Finished,
   Idle,
   Invalid,
   Ready,
   Running,
}

// IMPORTS //

use {
   crate::{
      external::*,
      memory::Stack,
      pointer::Unique,
      syscall::call_msb,
   },
   core::{
      cell::RefCell,
      fmt,
   },
   std_alloc::{
      boxed::Box,
      fmt::Display,
      rc::Rc,
   },
};
