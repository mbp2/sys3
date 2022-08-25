#ifndef _STD_MEM_H
#define _STD_MEM_H 1

#include <sys/cdefs.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif//__cplusplus

int MemCmp(const void*, const void*, size_t);
void* MemCpy(void* __restrict, const void* __restrict, size_t);
void* MemMove(void*, const void*, size_t);
void* MemSet(void*, int, size_t);

#ifdef __cplusplus
}
#endif//__cplusplus

#endif//_STD_MEM_H