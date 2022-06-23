#include "vga.h"

static inline uint8_t vga_color(enum VGA fg, enum VGA bg) {
  return fg | bg << 4;
}
 
static inline uint16_t vga_entry(unsigned char uc, uint8_t color) {
  return (uint16_t) uc | (uint16_t) color << 8;
}
