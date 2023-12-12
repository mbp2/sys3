pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn initialise() {
   unsafe {
      IDT.breakpoint.set_handler_fn(breakpoint);
      IDT.double_fault.set_handler_fn(double_fault);
      IDT.page_fault.set_handler_fn(page_fault);

      IDT.load();
   }

   log::info!("Added interrupt handlers to the IDT");
}

extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
   log::error!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, _: u64) -> ! {
   log::error!("EXCEPTION: DOUBLE FAULT\n{:#?}", frame);
   loop{}
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, code: PageFaultErrorCode) {
   use x86_64::registers::control::Cr2;

   log::error!("EXCEPTION: PAGE FAULT");
   log::error!("Accessed address: {:?}", Cr2::read());
   log::error!("Error code: {:?}", code);
   log::error!("{:#?}", frame);

   loop{}
}

// IMPORTS //

use {
   base::log,
   x86_64::structures::idt::{
      InterruptDescriptorTable, InterruptStackFrame,
      PageFaultErrorCode,
   }
};
