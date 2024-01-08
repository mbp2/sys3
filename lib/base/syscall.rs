/// number of the system call `write`
pub const SYSNO_WRITE: usize = 1;

/// number of the system call `close`
pub const SYSNO_CLOSE: usize = 3;

pub const SYSNO_IOCTL: usize = 16;

pub const SYSNO_WRITEV: usize = 20;

/// number of the system call `exit`
pub const SYSNO_EXIT: usize = 60;

pub const SYSNO_ARCH_PRCTL: usize = 158;

/// set pointer to thread ID
pub const SYSNO_SET_TID_ADDRESS: usize = 218;

/// exit all threads in a process
pub const SYSNO_EXIT_GROUP: usize = 231;

/// total number of system calls
pub const NUM_SYSCALLS: usize = 400;

pub fn make_syscall(call: usize) {}

#[naked]
#[no_mangle]
#[allow(unused_assignments)]
pub unsafe extern "C" fn sys_invalid() {
   asm!("mov rdi, rax",
   "call {}",
   sym invalid_syscall,
   options(noreturn),
   );
}

#[repr(C)]
#[repr(align(64))]
pub struct SyscallTable{
   pub handle: [*const usize; NUM_SYSCALLS],
}

impl SyscallTable {
   pub const fn new() -> Self {
      let mut table = SyscallTable{
         handle: [sys_invalid as *const _; NUM_SYSCALLS],
      };

      return table;
   }
}

// X86 SYSTEM CALLS //

/// This macro can be used to call system functions from user-space
#[macro_export]
macro_rules! syscall {
	($arg0:expr) => {
		$crate::arch::x86_64::syscall::syscall0($arg0 as u64)
	};

	($arg0:expr, $arg1:expr) => {
		$crate::arch::x86_64::syscall::syscall1($arg0 as u64, $arg1 as u64)
	};

	($arg0:expr, $arg1:expr, $arg2:expr) => {
		$crate::arch::x86_64::syscall::syscall2($arg0 as u64, $arg1 as u64, $arg2 as u64)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {
		$crate::arch::x86_64::syscall::syscall3($arg0 as u64, $arg1 as u64, $arg2 as u64, $arg3 as u64)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {
		$crate::arch::x86_64::syscall::syscall4(
			$arg0 as u64,
			$arg1 as u64,
			$arg2 as u64,
			$arg3 as u64,
			$arg4 as u64,
		)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr) => {
		$crate::arch::x86_64::syscall::syscall5(
			$arg0 as u64,
			$arg1 as u64,
			$arg2 as u64,
			$arg3 as u64,
			$arg4 as u64,
			$arg5 as u64,
		)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr) => {
		$crate::arch::x86_64::syscall::syscall6(
			$arg0 as u64,
			$arg1 as u64,
			$arg2 as u64,
			$arg3 as u64,
			$arg4 as u64,
			$arg5 as u64,
			$arg6 as u64,
		)
	};

	($arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr, $arg7:expr) => {
		$crate::arch::x86_64::syscall::syscall7(
			$arg0 as u64,
			$arg1 as u64,
			$arg2 as u64,
			$arg3 as u64,
			$arg4 as u64,
			$arg5 as u64,
			$arg6 as u64,
			$arg7 as u64,
		)
	};
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall0(arg0: u64) -> u64 {
   let mut ret: u64;
   unsafe {
      asm!("syscall",
      inlateout("rax") arg0 => ret,
      lateout("rcx") _,
      lateout("r11") _,
      options(preserves_flags, nostack)
      );
   }
   ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall1(arg0: u64, arg1: u64) -> u64 {
   let mut ret: u64;
   unsafe {
      asm!("syscall",
      inlateout("rax") arg0 => ret,
      in("rdi") arg1,
      lateout("rcx") _,
      lateout("r11") _,
      options(preserves_flags, nostack)
      );
   }
   ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall2(arg0: u64, arg1: u64, arg2: u64) -> u64 {
   let mut ret: u64;
   unsafe {
      asm!("syscall",
      inlateout("rax") arg0 => ret,
      in("rdi") arg1,
      in("rsi") arg2,
      lateout("rcx") _,
      lateout("r11") _,
      options(preserves_flags, nostack)
      );
   }
   ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall3(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
   let mut ret: u64;
   unsafe {
      asm!("syscall",
      inlateout("rax") arg0 => ret,
      in("rdi") arg1,
      in("rsi") arg2,
      in("rdx") arg3,
      lateout("rcx") _,
      lateout("r11") _,
      options(preserves_flags, nostack)
      );
   }
   ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall4(arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
   let mut ret: u64;
   unsafe {
      asm!("syscall",
      inlateout("rax") arg0 => ret,
      in("rdi") arg1,
      in("rsi") arg2,
      in("rdx") arg3,
      in("r10") arg4,
      lateout("rcx") _,
      lateout("r11") _,
      options(preserves_flags, nostack)
      );
   }
   ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall5(arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {
   let mut ret: u64;
   unsafe {
      asm!("syscall",
      inlateout("rax") arg0 => ret,
      in("rdi") arg1,
      in("rsi") arg2,
      in("rdx") arg3,
      in("r10") arg4,
      in("r8") arg5,
      lateout("rcx") _,
      lateout("r11") _,
      options(preserves_flags, nostack)
      );
   }
   ret
}

#[inline(always)]
#[allow(unused_mut)]
pub fn syscall6(
   arg0: u64,
   arg1: u64,
   arg2: u64,
   arg3: u64,
   arg4: u64,
   arg5: u64,
   arg6: u64,
) -> u64 {
   let mut ret: u64;
   unsafe {
      asm!("syscall",
      inlateout("rax") arg0 => ret,
      in("rdi") arg1,
      in("rsi") arg2,
      in("rdx") arg3,
      in("r10") arg4,
      in("r8") arg5,
      in("r9") arg6,
      lateout("rcx") _,
      lateout("r11") _,
      options(preserves_flags, nostack)
      );
   }
   ret
}


#[no_mangle]
pub extern "C" fn invalid_syscall(sys_no: u64) -> ! {
   log::error!("Invalid syscall: {}", sys_no);
   //crate::tasks::do_exit(); // TODO: uncomment when proper scheduling is added
   loop{
      unsafe{ asm!("hlt", options(nomem, nostack)); }
   }
}


// UTILITIES //

/// Forces strict CPU ordering, serialises load and store operations.
#[inline(always)]
pub fn call_mb() {
   unsafe{
      asm!("mfence", options(preserves_flags, nostack));
   }
}

/// Search the most-significant bit.
#[inline(always)]
pub fn call_msb(value: u64) -> Option<u64> {
   return if value > 0 {
      let ret: u64;
      unsafe{
         asm!("bsr {0}, {1}",
         out(reg) ret,
         in(reg) value,
         options(nomem, nostack)
         );
      }

      Some(ret)
   } else {
      None
   };
}

/// Search the least-significant bit.
#[inline(always)]
pub fn call_lsb(value: u64) -> Option<u64> {
   return if value > 0 {
      let ret: u64;
      unsafe{
         asm!("bsf {0}, {1}",
         out(reg) ret,
         in(reg) value,
         options(nomem, nostack)
         );
      }

      Some(ret)
   } else {
      None
   };
}

/// x86_64 `hlt` instruction.
#[inline(always)]
pub fn halt() {
   unsafe{
      asm!("hlt", options(nomem, nostack));
   }
}

/// x86_64 `pause` instruction.
#[inline(always)]
pub fn pause() {
   unsafe{
      asm!("pause", options(nomem, nostack));
   }
}

#[no_mangle]
pub extern "C" fn shutdown() -> ! {
   // TODO: implement a proper shutdown sequence.
   crate::println!("Shutting down...");
   loop{}
}

// IMPORTS //

use core::arch::asm;

// MODULES //

/// Implements a generic PIO interface.
pub mod pio;

// EXPORTS //

pub use self::pio::Pio;