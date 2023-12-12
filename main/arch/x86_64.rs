/// Build our system peripheral setup.
pub fn initialise_platform() {
   log::debug!("Detected x86_64 CPU!");
   log::debug!("Moving to initialise x86_64 platform modules.");

   COM2.lock().initialise();

   log::debug!("Initialise timer, PIT, et cetera.");

   let latch = ((CLOCK_TICK_RATE + TIMER_FREQUENCY / 2) / TIMER_FREQUENCY) as u16;

   unsafe {
      /*
       * Port 0x43 is for initializing the PIT:
       *
       * 0x34 means the following:
       * 0b...     (step-by-step binary representation)
       * ...  00  - channel 0
       * ...  11  - write two values to counter register:
       *            first low-, then high-byte
       * ... 010  - mode number 2: "rate generator" / frequency divider
       * ...   0  - binary counter (the alternative is BCD)
       */
      outb(0x43, 0x34);

      wait_100k();

      /* Port 0x40 is for the counter register of channel 0 */

      outb(0x40, (latch & 0xFF) as u8); /* low byte  */

      wait_100k();

      outb(0x40, (latch >> 8) as u8); /* high byte */
   }

   log::info!("Successfully initialised x86_64 platform modules.");
}

// IMPORTS //

use {
   self::{syscall::*, timer::*},
   base::{
      log,
      uart::COM2,
   },
   x86::io::*,
};

// MODULES //

pub mod syscall;
pub mod timer;
