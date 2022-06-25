.set ALIGN, 1<<0
.set MEMINFO, 1<<1
.set FLAGS, ALIGN | MEMINFO
.set MAGIC,    0x1BADB002
.set CHECKSUM, -(MAGIC + FLAGS)

.section .multiboot
.align 4
.long ALIGN
.long FLAGS
.long CHECKSUM

.section .bss
.align 16

stack_bottom:
.skip 16384 # We skip 16KB

stack_top:

.section .text
.global _start
.type _start, @function
_start:  
  /*
  To get our stack setup, we need to set the
  ESP register to the top of the stack.
  */
  mov $stack_top, %esp
  
  # Call into our main function
  call kernel_start
  cli

1:hlt
  jmp 1b

/*
Set the size of the _start symbol to the 
current location '.' minus its start.
*/
.size _start, . - _start
