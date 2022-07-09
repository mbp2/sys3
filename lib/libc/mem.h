#ifndef _MEM_H
#define _MEM_H 1

#include <sys/cdefs.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif//__cplusplus

int memcmp(const void*, const void*, size_t);
void* memcpy(void* __restrict, const void* __restrict, size_t);
void* memmove(void*, const void*, size_t);
void* memset(void*, int, size_t);

#ifdef __cplusplus
}
#endif//__cplusplus

#endif//_MEM_H