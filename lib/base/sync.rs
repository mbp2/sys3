/// `irqsave` guarantees that the call of the closure
/// will be not disturbed by an interrupt
#[inline]
pub fn irqsave<F, R>(f: F) -> R
   where
      F: FnOnce() -> R,
{
   let irq = irq_nested_disable();
   let ret = f();
   irq_nested_enable(irq);
   ret
}

// IMPORTS //

use crate::arch::irq::*;

// MODULES //

/// Implements an atomic spinlock primitive called [`SpinlockIrqSave`](crate::sync::spinlock::SpinlockIrqSave).
pub mod spinlock;

// EXPORTS //

pub use self::{
   spinlock::SpinlockIrqSave,
};
