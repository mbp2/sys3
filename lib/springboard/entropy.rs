/// Gather entropy from various sources to seed a RNG.
pub fn BuildRng() -> Hc128Rng {
   const ENTROPY_SOURCES: [fn() -> [u8; 32]; 3] = [rdRandEntropy, tscEntropy, pitEntropy];

   let mut seed = [0; 32];
   for src in ENTROPY_SOURCES {
      let entropy = src();

      for (seed, entropy) in seed.iter_mut().zip(entropy) {
         *seed ^= entropy;
      }
   }

   return Hc128Rng::from_seed(seed);
}

/// Try to fetch a 64 bit random value with a retry count limit of 10.
///
/// This function is a port of the C implementation provided in Intel's Software Developer's Manual, Volume 1, 7.3.17.1.
pub fn GetRandom64(rd: RdRand) -> Option<u64> {
   const RETRY_LIMIT: u32 = 10;

   for _ in 0..RETRY_LIMIT {
      if let Some(value) = rd.get_u64() {
         return Some(value);
      }
   }

   return None;
}

/// Gather entropy by requesting random numbers with `RDRAND` instruction if it's available.
///
/// This function provides excellent entropy (unless you don't trust the CPU vendors).
fn rdRandEntropy() -> [u8; 32] {
   let mut entropy = [0; 32];

   // Check if our CPU supports RDRAND
   if let Some(rdRand) = RdRand::new() {
      for i in 0..4 {
         if let Some(value) = GetRandom64(rdRand) {
            entropy[i * 8..(i + 1) * 8].copy_from_slice(&value.to_ne_bytes());
         }
      }
   }

   return entropy;
}

/// Gather entropy by reading the current time with the `RDTSC` instruction if it's available.
///
/// This function doesn't provide particularly good entropy, but it's better than nothing.
fn tscEntropy() -> [u8; 32] {
   let mut entropy = [0; 32];

   // Check if our CPU supports RDTSC
   let cpuID = CpuId::new();
   if let Some(feature_info) = cpuID.get_feature_info() {
      if !feature_info.has_tsc() {
         for i in 0..4 {
            let value = unsafe {
               // SAFETY: we have checked the CPU supports RDTSC and run in ring 0
               core::arch::x86_64::_rdtsc()
            };

            entropy[i * 8..(i + 1) * 8].copy_from_slice(&value.to_ne_bytes());
         }
      }
   }

   return entropy;
}

/// Gather entropy by reading the current count of PIT channel 1-3.
///
/// This function doesn't provide particularly good entropy, but it's always available.
fn pitEntropy() -> [u8; 32] {
   let mut entropy = [0; 32];

   for (i, entropy_byte) in entropy.iter_mut().enumerate() {
      // Cycle through channels 1-3.
      let channel = i % 3;

      let mut port = Port::<u8>::new(0x40 + channel as u16);
      let value = unsafe {
         // SAFETY: It's safe to read from ports 0x40-0x42.
         port.read()
      };

      *entropy_byte = value;
   }

   entropy
}

// IMPORTS //

use {
   rand::SeedableRng,
   rand_hc::Hc128Rng,
   raw_cpuid::CpuId,
   x86_64::instructions::{port::Port, random::RdRand},
};
