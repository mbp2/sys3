#include <libc/mem.h>

int memcmp(const void* p1, const void* p2, size_t size) {
   const unsigned char* a = (const unsigned char*)p1;
   const unsigned char* b = (const unsigned char*)p2;

   for (size_t i = 0; i < size; i++) {
      if (a[i] < b[i]) {
         return -1;
      } else if (b[i] < a[i]) {
         return 1;
      }
   }

   return 0;
}

void* memcpy(void* restrict dstPtr, const void* restrict srcPtr, size_t size) {
   unsigned char* dst = (unsigned char*)dstPtr;
   const unsigned char* src = (const unsigned char*)srcPtr;

   for (size_t i = 0; i < size; i++) {
      dst[i] = src[i];
   }

   return dstPtr;
}

void* memmove(void* dstPtr, const void* srcPtr, size_t size) {
   unsigned char* dst = (unsigned char*) dstPtr;
   const unsigned char* src = (const unsigned char*) srcPtr;

   if (dst < src) {
      for (size_t i = 0; i < size; i++) {
         dst[i] = src[i];
      }
   } else {
      for (size_t i = size; i != 0; i--) {
         dst[i-1] = src[i-1];
      }
   }

   return dstPtr;
}

void* memset(void* bufPtr, int value, size_t size) {
   unsigned char* buf = (unsigned char*) bufPtr;

   for (size_t i = 0; i < size; i++) {
      buf[i] = (unsigned char) value;
   }

   return bufPtr;
}
