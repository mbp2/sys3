pub const CLOCK_TICK_RATE: u32 = 1193182u32; // 8254 chip's internal oscillator frequency
pub const TIMER_FREQUENCY: u32 = 100; // Timer frequency in Hertz.

pub unsafe fn wait_100k() {
   let start = rdtsc();
   call_mb();

   while rdtsc() - start < 1000000 {
      call_mb();
   }
}

// IMPORTS //

use {
   base::syscall::*,
   x86::{time::rdtsc}
};
