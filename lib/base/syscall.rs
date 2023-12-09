pub fn make_syscall(call: usize) {}

// MODULES //

/// Implements a generic PIO interface.
pub mod pio;

// EXPORTS //

pub use self::pio::Pio;
