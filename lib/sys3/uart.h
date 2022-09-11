#ifndef LIBK_UART_H
#define LIBK_UART_H 1

#include<stddef.h>

typedef struct Uart {
   size_t base; // The base address
} Uart;

struct Uart* UartInit(size_t);
uint32_t UartRead(Uart*);
void UartWrite(Uart*, int);

#endif//LIBK_UART_H
