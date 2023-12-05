pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn initialise() {
   unsafe {
      IDT.breakpoint.set_handler_fn(breakpoint);
      IDT.double_fault.set_handler_fn(double_fault);
      IDT.page_fault.set_handler_fn(page_fault);

      IDT.load();
   }
}

extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
   println!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, _: u64) -> ! {
   println!("EXCEPTION: DOUBLE FAULT\n{:#?}", frame);
   loop{}
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, code: PageFaultErrorCode) {
   use x86_64::registers::control::Cr2;

   println!("EXCEPTION: PAGE FAULT");
   println!("Accessed address: {:?}", Cr2::read());
   println!("Error code: {:?}", code);
   println!("{:#?}", frame);

   loop{}
}

// IMPORTS //

use {
   x86_64::structures::idt::{
      InterruptDescriptorTable, InterruptStackFrame,
      PageFaultErrorCode,
   }
};
