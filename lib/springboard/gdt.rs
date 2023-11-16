/// Create and load the Global Descriptor Table into memory.
pub fn CreateAndLoadGdt(frame: PhysFrame) {
   let physAddress = frame.start_address();
   log::info!("Creating GDT at {:?}", physAddress);
   let virtAddress = VirtAddr::new(physAddress.as_u64());

   let ptr: *mut GlobalDescriptorTable = virtAddress.as_mut_ptr();

   let mut gdt = GlobalDescriptorTable::new();
   let codeSelector = gdt.add_entry(Descriptor::kernel_code_segment());
   let dataSelector = gdt.add_entry(Descriptor::kernel_data_segment());
   let gdt = unsafe {
      ptr.write(gdt);
      &*ptr
   };

   gdt.load();
   unsafe {
      segmentation::CS::set_reg(codeSelector);
      segmentation::DS::set_reg(dataSelector);
      segmentation::ES::set_reg(dataSelector);
      segmentation::SS::set_reg(dataSelector);
   }
}

// IMPORTS //

use x86_64::{
   instructions::segmentation::{self, Segment},
   structures::{
      gdt::{Descriptor, GlobalDescriptorTable},
      paging::PhysFrame,
   },
   VirtAddr,
};
