#ifndef SYS3_HEAP_H
#define SYS3_HEAP_H

void InitHeap(void*, size_t, FreeBlock**);

FreeBlock NewFreeBlock(FreeBlock*);

/// The interface to our heap.
///
/// It must be stored somewhere *OUTSIDE* of the heap as potentially every byte
/// is available for allocation.
typedef struct Heap {
   void* base;
   size_t size;
   size_t min_block_size;
   int8_t min_block_size_log2;
   FreeBlock** free_lists;
} Heap;

typedef struct FreeBlock {
   FreeBlock* next;
} FreeBlock;

#endif//SYS3_HEAP_H
