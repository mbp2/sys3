#include "vga.h"

static inline uint8_t VgaColour(enum VGA fg, enum VGA bg) {
   return fg | bg << 4;
}

static inline uint16_t VgaEntry(unsigned char uc, uint8_t colour) {
   return (uint16_t) uc | (uint16_t) colour << 8;
}
