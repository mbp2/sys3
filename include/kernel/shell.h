// A simplistic shell implementation.
//
// TODO: Write proper documentation.

#pragma once
#ifndef SHELL_H
#define SHELL_H

#include <string.h>

#include <stdint.h>
#include <stddef.h>

static const size_t SHELL_WIDTH = 80;
static const size_t SHELL_LENGTH = 25;

size_t sh_cols;
size_t sh_rows;
uint8_t sh_colour;
uint16_t* sh_buf;

void ShInit(void);
void ShSetColour(uint8_t);
void ShPutEntryAt(char, uint8_t, size_t, size_t); 
void ShPutChar(char);
void ShWrite(const char*, size_t); 
void ShWriteString(const char*);

#endif//SHELL_H
