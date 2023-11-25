pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn initIDT() {
   unsafe {
      IDT.breakpoint.set_handler_fn(breakpoint);
      IDT.double_fault.set_handler_fn(double_fault);

      IDT.load();
   }
}

extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, code: u64) -> ! {
   loop{}
}

// IMPORTS //

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
