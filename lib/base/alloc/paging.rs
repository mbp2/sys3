pub static mut ALLOC_START: usize = 0;
pub const PAGE_SIZE: usize = 1 << PAGE_ORDER;
pub const PAGE_ORDER: usize = 12;

/// Represent (repr) our entry bits as
/// unsigned 64-bit integers.
#[repr(usize)]
#[derive(Copy, Clone)]
pub enum EntryBits {
   None = 0,
   Valid = 1 << 0,
   Read = 1 << 1,
   Write = 1 << 2,
   Execute = 1 << 3,
   User = 1 << 4,
   Global = 1 << 5,
   Access = 1 << 6,
   Dirty = 1 << 7,

   // Convenience combinations
   ReadWrite = 1 << 1 | 1 << 2,
   ReadExecute = 1 << 1 | 1 << 3,
   ReadWriteExecute = 1 << 1 | 1 << 2 | 1 << 3,

   // User Convenience Combinations
   UserReadWrite = 1 << 1 | 1 << 2 | 1 << 4,
   UserReadExecute = 1 << 1 | 1 << 3 | 1 << 4,
   UserReadWriteExecute = 1 << 1 | 1 << 2 | 1 << 3 | 1 << 4,
}

impl EntryBits {
   pub fn value(self) -> usize {
      return self as usize;
   }
}

#[derive(Clone, Copy, Debug)]
pub struct Entry {
   pub entry: usize,
}

impl Entry {
   pub fn valid(&self) -> bool {
      return self.entry & EntryBits::Valid.value() != 0;
   }

   // The first bit (bit index #0) is the V bit for
   // valid.
   pub fn invalid(&self) -> bool {
      return !self.valid();
   }

   // A leaf has one or more RWX bits set
   pub fn leaf(&self) -> bool {
      return self.entry & 0xe != 0;
   }

   pub fn branch(&self) -> bool {
      return !self.leaf();
   }

   pub fn set_entry(&mut self, entry: usize) {
      self.entry = entry;
   }
}

#[derive(Clone, Copy, Debug)]
pub struct Table {
   pub entries: [Entry; 512],
}

impl Table {
   pub fn length() -> usize {
      return 512;
   }
}

// BUDDY ALLOCATOR //

// BEST-FIT ALLOCATOR //
// TODO: implement best-fit allocator.

// IMPORTS //
