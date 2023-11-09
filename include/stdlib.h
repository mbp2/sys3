#ifndef LIBC_STDLIB_H
#define LIBC_STDLIB_H 1
 
#include <libc/sys/cdefs.h>
 
#ifdef __cplusplus
extern "C" {
#endif
 
__attribute__((__noreturn__))
void abort(void);
 
#ifdef __cplusplus
}
#endif
 
#endif//LIBC_STDLIB_H