#include <stdio.h>
#include <stdlib.h>
 
__attribute__((__noreturn__))
void abort(void) {
#if defined(__is_libKernel)
   // TODO: Add proper kernel panic.
   Printf("kernel: panic: abort()\n");
   asm volatile("hlt");
#else
   // TODO: Abnormally terminate the process as if by SIGABRT.
   Printf("abort()\n");
#endif
   while (1) { }
   __builtin_unreachable();
}