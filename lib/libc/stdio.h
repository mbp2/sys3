#ifndef _STDIO_H
#define _STDIO_H 1
 
#include <sys/cdefs.h>
 
#define EOF (-1)
 
#ifdef __cplusplus
extern "C" {
#endif

int Printf(const char* __restrict, ...);
int PutChar(int);
int Puts(const char*);
 
#ifdef __cplusplus
}
#endif
 
#endif