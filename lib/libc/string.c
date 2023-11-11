#include <string.h>

union intptr {
    char* c;
    long* l;
#define LSIZE sizeof(long)
};

#define aligned_(x, a) \
    ((unsigned long) (x) % (a) == 0)

#define punpktt_(x, from, to) \
    ((to) (-1)/(from) (-1)*(from) (x))
#define punpkbl_(x) \
    punpktt_(x, unsigned char, unsigned long)

#define plessbl_(x, y) \
    (((x) - punpkbl_(y)) & ~(x) & punpkbl_(0x80))
#define pzerobl_(x) \
    plessbl_(x, 1)

static inline unsigned long maskffs_(unsigned long x)
{
    unsigned long acc = 0x00010203UL;
    if (LSIZE == 8)
       acc = ((acc << 16) << 16) | 0x04050607UL;
    return ((x & -x) >> 7) * acc >> (LSIZE*8-8);
}

size_t strlen(const char* base) {
   union intptr p = { (char*) base };
   unsigned long mask;

   for ( ; !aligned_(p.c, LSIZE); p.c++ )
      if (*p.c == 0)
         return p.c - base;

   while ( !(mask = pzerobl_(*p.l)) )
      p.l++;

   return p.c - base + maskffs_(mask);
}
