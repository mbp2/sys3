#include "shell.h"

void kernelMain(void) {
   sh_init();

   sh_writeString("Hello world!");
   return;
}
