/// A helper type used to offset virtual addresses for position independent executables.
#[derive(Copy, Clone)]
pub struct VirtualAddressOffset(pub i128);

impl VirtualAddressOffset {
   pub fn zero() -> Self {
      return VirtualAddressOffset::new(0);
   }

   pub fn new(offset: i128) -> Self {
      return VirtualAddressOffset(offset);
   }

   pub fn Offset(&self) -> i128 {
      return self.0;
   }
}

impl Add<u64> for VirtualAddressOffset {
   type Output = u64;

   fn add(self, offset: u64) -> Self::Output {
      return u64::try_from(self.0.checked_add(i128::from(offset)).unwrap()).unwrap();
   }
}
