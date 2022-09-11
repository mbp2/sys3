#include "uart.h"

Uart* UartInit(size_t base) {
   uint8_t* ptr = (uint8_t*)base;

   size_t lcr = (1 << 0) | (1 << 1);
   *(ptr + 3) = lcr;

   *(ptr + 2) = (1 << 0);

   *(ptr + 1) = (1 << 0);

   uint16_t divisor = 592;
   uint8_t divLeast = divisor & 0xff;
   uint8_t divMost  = divisor >> 8;

   *(ptr + 3) = (lcr | 1 << 7);

   *(ptr + 0) = divLeast;
   *(ptr + 1) = divMost;

   *(ptr + 3) = lcr;

   return Uart{ base };
}

uint32_t UartRead(Uart* self, uint32_t offset) {
   uint8_t* volatile ptr = (uint8_t*)self->base;
   
   return *(ptr + offset);
}

void UartWrite(Uart* self, uint32_t offset, uint8_t value) {
   uint8_t* volatile ptr = (uint8_t*)self->base;
   
   *(ptr + offset) = value;
}