#include <kernel/uart.h>

struct Uart* UartInit(size_t base) {
   struct Uart* uart = { base };
   return uart;
}

uint32_t UartRead(Uart* self, uint32_t offset) {
   uint8_t* volatile ptr = (uint8_t*)self->base;
   
   return *(ptr + offset);
}

void UartWrite(Uart* self, uint32_t offset, uint8_t value) {
   uint8_t* volatile ptr = (uint8_t*)self->base;
   
   *(ptr + offset) = value;
}