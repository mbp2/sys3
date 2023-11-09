#include <kernel/uart.h>

void Main(void) {
   ShInit();

   ShWriteString("Hello world!");
   return;
}
