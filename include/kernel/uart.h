#ifndef UART_H
#define UART_H 1

#include<stddef.h>
#include<stdint.h>

typedef struct Uart {
   size_t base; // The base address
} Uart;

Uart* UartInit(size_t);
uint32_t UartRead(Uart*, uint32_t);
void UartWrite(Uart*, uint32_t, uint8_t);

#endif//UART_H
