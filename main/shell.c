#include <kernel/vga.h>
#include <kernel/shell.h>

void ShInit(void) {
   sh_cols = 0;
   sh_rows = 0;
   sh_colour = VgaColour(VGA_LIGHT_GREY, VGA_BLACK);
   sh_buf = (uint16_t*) 0xB8000;

   for (size_t y = 0; y < SHELL_LENGTH; y++) {
      for (size_t x = 0; x < SHELL_WIDTH; x++) {
         const size_t index = y * SHELL_WIDTH + x;
         sh_buf[index] = VgaEntry(' ', sh_colour);
      }
   }
}

void ShSetColour(uint8_t colour) {
   sh_colour = colour;
}

void ShPutEntryAt(char c, uint8_t colour, size_t x, size_t y) {
   const size_t index = y * SHELL_WIDTH + x;
   sh_buf[index] = VgaEntry(c, colour);
}

void ShPutChar(char c) {
   ShPutEntryAt(c, sh_colour, sh_cols, sh_rows);

   if (++sh_cols == SHELL_WIDTH) {
      sh_cols = 0;

      if (++sh_rows == SHELL_LENGTH) {
         sh_rows = 0;
      }
   }
}

void ShWrite(const char* data, size_t size) {
   for (size_t i = 0; i < size; i++) {
      ShPutChar(data[i]);
   }
}

void ShWriteString(const char* data) {
   ShWrite(data, strlen(data));
}
