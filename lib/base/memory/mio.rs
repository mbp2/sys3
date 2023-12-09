/// # Safety
///
/// We label the mmio function unsafe since
/// we will be working with raw memory. Rust cannot
/// make any guarantees when we do this.
pub unsafe fn write(address: usize, offset: usize, value: u8) {
   // Set the pointer based off of the address
   let register = address as *mut u8;

   // write_volatile is a member of the *mut raw
   // and we can use the .add() to give us another pointer
   // at an offset based on the original pointer's memory
   // address. NOTE: The add uses pointer arithmetic so it is
   // new_pointer = old_pointer + sizeof(pointer_type) * offset
   register.add(offset).write_volatile(value);
}

/// # Safety
///
/// We label the mmio function unsafe since
/// we will be working with raw memory. Rust cannot
/// make any guarantees when we do this.
pub unsafe fn read(address: usize) -> Optional<u8> {
   // Set the pointer based off of the address
   let register = address as *mut u8;

   // read_volatile() is much like write_volatile() except it
   // will grab 8-bits from the pointer and give that value to us.
   return if register.add(5).read_volatile() & 1 == 0 {
      // No data.
      Optional::None
   } else {
      // Data found!
      Optional::Some(register.add(0).read_volatile())
   };
}

// IMPORTS //

use crate::optional::Optional;
