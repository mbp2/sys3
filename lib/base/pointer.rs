// MODULES //

/// Implements an allocator-aware smart pointer called [`Shared`](crate::pointer::shared::Shared)
pub mod shared;

/// Implements an allocator-aware smart pointer called [`Unique`](crate::pointer::unique::Unique).
pub mod unique;

// EXPORTS //

pub use self::unique::Unique;
