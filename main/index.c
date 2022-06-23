#include "shell.h"

void kernelMain(void) {
   shell_init();

   shell_writeString("Hello world!\n");
   return;
}
