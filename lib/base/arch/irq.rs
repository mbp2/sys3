/// Enable interrupts.
pub fn irq_enable() {
   unsafe{
      asm!("sti", options(preserves_flags, nomem, nostack));
   }
}

/// Disable interrupts.
pub fn irq_disable() {
   unsafe{
      asm!("cli", options(preserves_flags, nomem, nostack));
   }
}

/// Determines, if the interrupt flags (IF) is set
pub fn is_irq_enabled() -> bool {
   let rflags: u64;
   unsafe{ asm!("pushf; pop {}", lateout(reg) rflags, options(nomem, nostack, preserves_flags)) };

   if (rflags & (1u64 << 9)) != 0 {
      return true;
   }

   return false;
}

/// Disable IRQs (nested)
///
/// Disable IRQs when unsure if IRQs were enabled at all.
/// This function together with irq_nested_enable can be used
/// in situations when interrupts shouldn't be activated if they
/// were not activated before calling this function.
pub fn irq_nested_disable() -> bool {
   let was_enabled = is_irq_enabled();
   irq_disable();
   was_enabled
}

/// Enable IRQs (nested)
///
/// Can be used in conjunction with irq_nested_disable() to only enable
/// interrupts again if they were enabled before.
pub fn irq_nested_enable(was_enabled: bool) {
   if was_enabled == true {
      irq_enable();
   }
}

#[inline(always)]
pub fn send_eoi_to_slave() {
   /*
    * If the IDT entry that was invoked was greater-than-or-equal to 40
    * and lower than 48 (meaning IRQ8 - 15), then we need to
    * send an EOI to the slave controller of the PIC
    */
   unsafe {
      outb(0xA0, 0x20);
   }
}

#[inline(always)]
pub fn send_eoi_to_master() {
   /*
    * In either case, we need to send an EOI to the master
    * interrupt controller of the PIC, too
    */
   unsafe {
      outb(0x20, 0x20);
   }
}

// IMPORTS //

use {
   crate::{
      process::*,
      sync::spinlock::*,
   },
   core::{arch::asm, fmt},
   x86::io::*,
};
