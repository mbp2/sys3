macro_rules! save_context {
	() => {
		concat!(
			r#"
			pushfq
			push rax
			push rcx
			push rdx
			push rbx
			sub  rsp, 8
			push rbp
			push rsi
			push rdi
			push r8
			push r9
			push r10
			push r11
			push r12
			push r13
			push r14
			push r15
			"#,
		)
	};
}

macro_rules! restore_context {
	() => {
		concat!(
			r#"
			pop r15
			pop r14
			pop r13
			pop r12
			pop r11
			pop r10
			pop r9
			pop r8
			pop rdi
			pop rsi
			pop rbp
			add rsp, 8
			pop rbx
			pop rdx
			pop rcx
			pop rax
			popfq
			ret
			"#
		)
	};
}

#[naked]
pub unsafe extern "C" fn switch(_old_stack: *mut usize, _new_stack: usize) {
   // rdi = old_stack => the address to store the old rsp
   // rsi = new_stack => stack pointer of the new task

   asm!(
		save_context!(),
		"rdfsbase rax",
		"rdgsbase rdx",
		"push rax",
		"push rdx",
		// Store the old `rsp` behind `old_stack`
		"mov [rdi], rsp",
		// Set `rsp` to `new_stack`
		"mov rsp, rsi",
		// Set task switched flag
		"mov rax, cr0",
		"or rax, 8",
		"mov cr0, rax",
		// set stack pointer in TSS
		"call {set_stack}",
		"pop r15",
		"wrgsbase r15",
		"pop r15",
		"wrfsbase r15",
		restore_context!(),
		set_stack = sym set_current_kernel_stack,
		options(noreturn)
	);
}

#[inline(always)]
unsafe fn set_kernel_stack(stack: usize) {
	TSS.interrupt_stack_table[0] = VirtAddr::new(stack as u64);
}

#[no_mangle]
pub unsafe extern "C" fn set_current_kernel_stack() {
   let root = process::get_root_page_table() as u64;

   if root != cr3() {
      cr3_write(root);
   }

   let rsp = process::get_current_stack();
   set_kernel_stack(rsp + STACK_SIZE - 0x10);
}

// IMPORTS //

use {
	self::gdt::TSS,
	crate::{
		external::*,
		process,
	},
	core::arch::asm,
	x86::{
		controlregs::*,
		segmentation::*
	},
	x86_64::VirtAddr,
};

// MODULES //

/// The Global Descriptor Table (GDT) is a relic that was used for memory segmentation before
/// paging became the de facto standard. However, it is still needed in 64-bit mode for various
/// things, such as kernel/user mode configuration or TSS loading.
pub mod gdt;
