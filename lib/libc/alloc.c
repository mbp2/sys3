#include <libc/alloc.h>

#define MAX_ALLOC_ALLOWED 20

MallocInfo metadata_info[MAX_ALLOC_ALLOWED] = {0};

void* malloc(size_t size) {
}
