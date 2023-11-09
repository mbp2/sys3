<<<<<<< HEAD:lib/libc/mem.h
#ifndef LIBC_MEM_H
#define LIBC_MEM_H 1
=======
#ifndef _STD_MEM_H
#define _STD_MEM_H 1
>>>>>>> post-2:include/stdmem.h

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

<<<<<<< HEAD:lib/libc/mem.h
#endif//LIBC_MEM_H
=======
#endif//_STD_MEM_H
>>>>>>> post-2:include/stdmem.h
