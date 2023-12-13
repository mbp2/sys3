pub static mut SCHEDULER: Option<Scheduler> = None;

pub fn add_kernel_process(pid: u16) {}

pub fn register_task() {
   let sel: u16 = 6 << 3;

   unsafe{
      asm!("ltr ax", in("ax") sel, options(nostack, nomem));
   }
}

pub fn spawn(func: extern "C" fn(), priority: TaskPriority) -> Result<TaskId, ProcError> {
   Err(ProcError::NoStart)
}

/// Trigger the scheduler to switch to the next available task
pub fn reschedule() {
   unsafe { SCHEDULER.as_mut().unwrap().reschedule() }
}

/// Timer interrupt  call scheduler to switch to the next available task
pub fn schedule() {
   unsafe { SCHEDULER.as_mut().unwrap().schedule() }
}

/// Terminate the current running task
pub fn do_exit() -> ! {
   unsafe {
      SCHEDULER.as_mut().unwrap().exit();
   }
}

/// Terminate the current running task
pub fn abort() -> ! {
   unsafe { SCHEDULER.as_mut().unwrap().abort() }
}

pub fn get_current_stack() -> usize {
   unsafe { SCHEDULER.as_mut().unwrap().get_current_stack() }
}

pub fn get_root_page_table() -> usize {
   unsafe { SCHEDULER.as_mut().unwrap().get_root_page_table() }
}

pub fn set_root_page_table(addr: usize) {
   unsafe {
      SCHEDULER.as_mut().unwrap().set_root_page_table(addr);
   }
}

pub fn block_current_task() -> Rc<RefCell<Task>> {
   unsafe { SCHEDULER.as_mut().unwrap().block_current_task() }
}

pub fn wakeup_task(task: Rc<RefCell<Task>>) {
   unsafe { SCHEDULER.as_mut().unwrap().wakeup_task(task) }
}

/// Get the TaskID of the current running task
pub fn get_current_task_id() -> TaskId {
   unsafe { SCHEDULER.as_ref().unwrap().get_current_taskid() }
}

// IMPORTS //

use {
   self::{
      control::{Task, TaskId, TaskPriority},
      error::ProcError,
      scheduler::Scheduler,
   },
   crate::syscall::*,
   core::{
      arch::asm,
      cell::RefCell,
      ptr::null_mut,
   },
   spinning_top::Spinlock,
   std_alloc::{
      collections::{vec_deque::VecDeque, BTreeMap},
      rc::Rc,
      string::String,
   },
};

// MODULES //

/// Task control block.
pub mod control;

/// Task/process-related error handling.
pub mod error;

/// Interface to the process scheduler.
pub mod scheduler;
