#include <kernel/shell.h>

extern void kernel_main(void) {
   ShInit();

   ShWriteString("Hello world!");
   return;
}
