#ifndef LIBC_ALLOC_H
#define LIBC_ALLOC_H 1

#define TRUE 1
#define FALSE 0

#include <stdint.h>

typedef struct info_t {
  int32_t address;
  int32_t size;
} AllocInfo;

void* malloc(size_t);
void free(void*);
void* realloc(void*, size_t);
void* calloc(size_t, size_t);

#endif//LIBC_ALLOC_H