#ifndef LIBC_STRING_H
#define LIBC_STRING_H 1

#include <libc/sys/cdefs.h>
#include <stddef.h>

#ifdef  __cplusplus
extern "C" {
#endif//__cplusplus

size_t strlen(const char*);

#ifdef  __cplusplus
}
#endif//__cplusplus

#endif//_STRING_H