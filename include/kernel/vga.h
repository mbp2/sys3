// VGA COLOR CODES
//
// Defined in this file is an enum and a couple of functions.

<<<<<<<< HEAD:lib/sys3/shell/vga.h
#ifndef SYS3_SHELL_VGA_H
#define SYS3_SHELL_VGA_H 1
========
#ifndef VGA_H
#define VGA_H
>>>>>>>> post-2:include/kernel/vga.h

#include<stdint.h>
 
typedef enum vga {
   VGA_BLACK = 0,
   VGA_BLUE = 1,
   VGA_GREEN = 2,
   VGA_CYAN = 3,
   VGA_RED = 4,
   VGA_MAGENTA = 5,
   VGA_BROWN = 6,
   VGA_LIGHT_GREY = 7,
   VGA_DARK_GREY = 8,
   VGA_LIGHT_BLUE = 9,
   VGA_LIGHT_GREEN = 10,
   VGA_LIGHT_CYAN = 11,
   VGA_LIGHT_RED = 12,
   VGA_LIGHT_MAGENTA = 13,
   VGA_LIGHT_BROWN = 14,
   VGA_WHITE = 15,
} VGA;

<<<<<<<< HEAD:lib/sys3/shell/vga.h
static inline uint8_t VgaColour(enum VGA, enum VGA);
static inline uint16_t VgaEntry(unsigned char, uint8_t);
========
static inline uint8_t VgaColour(int fg, int bg) {
  return fg | bg << 4;
}
 
static inline uint16_t VgaEntry(unsigned char uc, uint8_t colour) {
  return (uint16_t) uc | (uint16_t) colour << 8;
}
>>>>>>>> post-2:include/kernel/vga.h

#endif//SYS3_SHELL_VGA_H
