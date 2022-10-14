#ifndef LIBC_ALLOC_H
#define LIBC_ALLOC_H 1

#include <stddef.h>

void* Alloc(size_t size);
void Free(void* pointer);
void* Realloc(void* pointer, size_t size);
void* Calloc(size_t count, size_t size);

#endif//LIBC_ALLOC_H