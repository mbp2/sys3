// MODULES //

/// A simple heap based on a buddy allocator.  For the theory of buddy
/// allocators, see https://en.wikipedia.org/wiki/Buddy_memory_allocation
///
/// The basic idea is that our heap size is a power of two, and the heap
/// starts out as one giant free block.  When a memory allocation request
/// is received, we round the requested size up to a power of two, and find
/// the smallest available block we can use.  If the smallest free block is
/// too big (more than twice as big as the memory we want to allocate), we
/// split the smallest free block in half recursively until it's the right
/// size.  This simplifies a lot of bookkeeping, because all our block
/// sizes are a power of 2, which makes it easy to have one free array per
/// block size.
pub mod heap;

/// Implements a simple memory layout structure.
pub mod layout;

/// Implements two memory allocators: a Buddy Allocator and a Best-Fit Allocator,
/// respectively referred to as [`BUDDY_ALLOCATOR`][crate::alloc::paging::BUDDY_ALLOCATOR]
/// and [`FIT_ALLOCATOR`][crate::alloc::paging::FIT_ALLOCATOR].
pub mod paging;

// EXPORTS //

pub use self::{
   layout::Layout,
};
