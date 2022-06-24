// A simplistic shell implementation.
//
// TODO: Write proper documentation.

#pragma once
#ifndef SHELL_H
#define SHELL_H

#include <stdint.h>

static const size_t SHELL_WIDTH = 80;
static const size_t SHELL_LENGTH = 25;

size_t sh_cols;
size_t sh_rows;
uint8_t sh_color;
uint16_t* sh_buf;

void sh_init(void);
void sh_setColor(uint8_t color);
void sh_putEntryAt(char c, uint8_t color, size_t x, size_t y); 
void sh_putChar(char c);
void sh_write(const char* data, size_t size); 
void sh_writeString(const char* data);

#endif//SHELL_H
