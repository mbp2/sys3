#include "shell.h"

extern void kernel_main(void) {
   sh_init();

   sh_writeString("Hello world!");
   return;
}
