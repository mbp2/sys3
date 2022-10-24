// VGA COLOR CODES
//
// Defined in this file is an enum and a couple of functions.

#ifndef SYS3_SHELL_VGA_H
#define SYS3_SHELL_VGA_H 1

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

static inline uint8_t VgaColour(enum VGA fg, enum VGA bg) {
   return fg | bg << 4;
}

static inline uint16_t VgaEntry(unsigned char uc, uint8_t color) {
   return (uint16_t) uc | (uint16_t) color << 8;
}


#endif//SYS3_SHELL_VGA_H
