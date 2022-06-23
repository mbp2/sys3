// A simplistic shell implementation.

#pragma once
#ifndef SHELL_H
#define SHELL_H

#include "defs.h"
#include "vga.h"

static const size_t SHELL_WIDTH = 80;
static const size_t SHELL_LENGTH = 25;

typedef struct shell {
   size_t cols;
   size_t rows;
   uint8_t color;
   uint16_t* buf;
} Shell;

struct Shell* sh;

void shell_init(void) {
   sh = &Shell{
      0, 
      0,
      vga_color(VGA_LIGHT_GREY, VGA_BLACK),
      (uint16_t*) 0xB8000,
   };

   for (size_t y = 0; y < VGA_HEIGHT; y++) {
      for (size_t x = 0; x < VGA_WIDTH; x++) {
         const size_t index = y * VGA_WIDTH + x;
         sh->buf[index] = vga_entry(' ', sh->color);
      }
   }
}

void shell_setColor(uint8_t color) {
   sh->color = color;
}

void shell_putEntryAt(char c, uint8_t color, size_t x, size_t y) {
  const size_t index = y * VGA_WIDTH + x;
  sh->buf[index] = vga_entry(c, color);
}
 
void shell_putChar(char c) {
  shell_putEntryAt(c, sh->color, sh->col, sh->row);
  
  if (++sh->col == VGA_WIDTH) {
    sh->col = 0;
    
    if (++sh->row == VGA_HEIGHT)
      sh->row = 0;
  }
}


void shell_write(const char* data, size_t size) {
  for (size_t i = 0; i < size; i++)
    shell_putChar(data[i]);
}
 
void shell_writeString(const char* data) {
  shell_write(data, strlen(data));
}

#endif//SHELL_H