#ifndef LIBK_UART_H
#define LIBK_UART_H 1

#include<stddef.h>
#include<stdint.h>

typedef struct Uart {
   size_t base; // The base address
} Uart;

<<<<<<<< HEAD:lib/sys3/uart.h
struct Uart* UartInit(size_t);
uint32_t UartRead(Uart*);
void UartWrite(Uart*, int);
========
Uart* UartInit(size_t);
uint32_t UartRead(Uart*, uint32_t);
void UartWrite(Uart*, uint32_t, uint8_t);
>>>>>>>> post-2:include/kernel/uart.h

#endif//LIBK_UART_H
