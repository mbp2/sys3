/// 64-bit ARM architecture-specific code.
///
/// Here, we initialise peripherals such as the COM2 and USB interfaces, map the pixel-buffer, and
/// set up our logging subsystem, and perform other hardware-level setup.
#[cfg(any(target_arch="riscv32", target_arch="riscv64"))]
pub mod riscv;

#[cfg(target_arch="aarch64")]
pub mod aarch;

/// RISC-V architecture-specific code.
///
/// Here, we initialise peripherals such as the COM2 and USB interfaces, map the pixel-buffer, and
/// set up our logging subsystem, and perform other hardware-level setup.
#[cfg(any(target_arch="riscv32", target_arch="riscv64"))]
pub mod riscv;


/// x86(_64) architecture-specific code.
///
/// Here, we initialise peripherals such as the COM2 and USB interfaces, map the pixel-buffer, and
/// set up our logging subsystem, and perform other hardware-level setup.
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
pub mod x86_64;
