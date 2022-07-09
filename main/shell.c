#include "shell.h"
#include "vga.h"

void sh_init(void) {
   sh_cols = 0;
   sh_rows = 0;
   sh_color = vga_color(VGA_LIGHT_GREY, VGA_BLACK);
   sh_buf = (uint16_t*) 0xB8000;

   for (size_t y = 0; y < VGA_HEIGHT; y++) {
      for (size_t x = 0; x < VGA_WIDTH; x++) {
         const size_t index = y * VGA_WIDTH + x;
         sh_buf[index] = vga_entry(' ', sh_color);
      }
   }
}

void sh_setColor(uint8_t color) {
   sh_color = color;
}

void sh_putEntryAt(char c, uint8_t color, size_t x, size_t y) {
   const size_t index = y * VGA_WIDTH + x;
   sh_buf[index] = vga_entry(c, color);
}

void sh_putChar(char c) {
   sh_putEntryAt(c, sh_color, sh_cols, sh_rows);

   if (++sh_cols == VGA_WIDTH) {
      sh_cols = 0;

      if (++sh_rows == VGA_HEIGHT) {
         sh_rows = 0;
      }
   }
}

void sh_write() {
   for (size_t i = 0; i < size; i++) {
      sh_putChar(data[i]);
   }
}

void shell_writeString(const char* data) {
   sh_write(data, strlen(data));
}
