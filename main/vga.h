// VGA COLOR CODES
//
// Defined in this file is an enum and a couple of functions.

#pragma once
#ifndef VGA_H
#define VGA_H

#include "defs.h"
 
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

static inline uint8_t vga_color(enum VGA fg, enum VGA bg);
static inline uint16_t vga_entry(unsigned char uc, uint8_t color);

#endif//VGA_H