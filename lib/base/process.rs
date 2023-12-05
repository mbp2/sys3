pub fn add_kernel_process(pid: u16) {}

// IMPORTS //

use {
   crate::syscall::*,
   core::ptr::null_mut,
   spinning_top::Spinlock,
   std_alloc::{
      string::String,
      collections::{vec_deque::VecDeque, BTreeMap},
   },
};
