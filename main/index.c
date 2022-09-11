#include <sys3/uart.h>

void KernelMain(void) {
   struct Uart* u = UartInit(0x100000000);
}
