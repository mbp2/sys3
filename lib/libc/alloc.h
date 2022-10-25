#ifndef LIBC_ALLOC_H
#define LIBC_ALLOC_H 1

#define TRUE 1
#define FALSE 0

#include <stddef.h>

typedef struct AllocInfo {
  int32_t addr;
  int32_t size;
} AllocInfo;

void* malloc(size_t size);
void free(void* pointer);
void* realloc(void* pointer, size_t size);
void* calloc(size_t count, size_t size);

#endif//LIBC_ALLOC_H