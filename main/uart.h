#ifndef UART_H
#define UART_H 1

#include<stddef.h>

typedef struct Uart {
   size_t base; // The base address
} Uart;

Uart* UartInit(size_t);
uint32_t UartRead(Uart*);
void UartWrite(Uart*, int);

#endif//UART_H
