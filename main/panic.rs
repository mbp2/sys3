/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
   loop {}
}

// IMPORTS //

use core::panic::PanicInfo;
