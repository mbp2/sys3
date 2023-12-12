pub trait Stack {
   fn top(&self) -> usize;
   fn bottom(&self) -> usize;
}

// MODULES //

pub mod mio;
pub mod offset;

// EXPORTS //

pub use self::offset::VirtualAddressOffset;
