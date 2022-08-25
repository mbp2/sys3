#include "uart.h"

Uart* UartInit(size_t base) {
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