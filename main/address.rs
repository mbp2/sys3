#![allow(dead_code)]

/// Define the size of the kernel stack.
pub const STACK_SIZE: usize = 0x4000;

/// Size of a cache line.
pub const CACHE_LINE: usize = 64;

/// Maximum number of priorities.
pub const NUM_PRIORITIES: usize = 32;

/// Start address of user space.
pub const USER_SPACE_START: usize = 0x20000000000usize;

/// Initial value of the stack pointer.
pub const USER_STACK: usize = USER_SPACE_START + 0x800000000usize;
