#include <kernel/uart.h>
#include<limits.h>
#include<stdarg.h>
#include<stdbool.h>

#include "stdio.h"
#include "string.h"

int PutChar(int cc) {
#if defined(__is_libKernel)
   u = UartInit(0x100000000)
   char c = (char)cc;
   UartWrite(u, &c, sizeof(c));
#else
   // WE'LL COME BACK *HERE* LATER
#endif
   
   return cc;
}

static bool Print(const char* data, size_t len) {
   const unsigned char* bytes = (const unsigned char*)data;
   
   for (size_t i = 0; i<len; i++) {
      if (PutChar(bytes[i]) == EOF) {
         return false;
      }
   }
   
   return true;
}

int Printf(const char* restrict fmt, ...) {
   va_list params;
   va_start(params, fmt);
   
   int written = 0;
   
   while(*fmt != '\0') {
      size_t max = INT_MAX - written;
      
      if(fmt[0] != '%' || fmt[1] == '%') {
         if(fmt[0] == '%') {
            fmt++;
         }
         
         size_t amount = 1;
         
         while (fmt[amount] && fmt[amount] != '%') {
            amount++;
         }
         
         if (max < amount) {
            // We'll have to come back here and set errno to EOVERFLOW.
            return -1;
         }
         
         if (!Print(fmt, amount)) {
            return -1;
         }
         
         fmt += amount;
         written += amount;
         
         continue;
      }
      
      const char* fmt_started = fmt++;
      
      if(*fmt == 'c') {
         fmt++;
         char c = (char)va_arg(params, int /* char promotes to int */);
         
         if(!max) {
            // We'll have to come back here and set errno to EOVERFLOW.
            return -1;
         }
         
         if(!Print(&c, sizeof(c))) {
            return -1;
         }
         
         written++;
      } else if(*fmt == 's') {
         fmt++;
         const char* str = va_arg(params, const char*);
         size_t len = strlen(str);
         
         if(max < len) {
            // We'll have to come back here and set errno to EOVERFLOW.
            return -1;
         }
         
         if(!Print(str, len)) {
            return -1;
         }
         
         written += len;
      } else {
         fmt = fmt_started;
         size_t len = strlen(fmt);
         
         if(max < len) {
            // We'll have to come back here and set errno to EOVERFLOW.
            return -1;
         }
         
         if(!Print(fmt, len)) {
            return -1;
         }
         
         written += len;
         fmt += len;
      }
   }
   
   va_end(params);
   return written;
}

int Puts(const char* string) { return Printf("%s\n", string); }