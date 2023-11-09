#include<stdint.h>

typedef struct __attribute__((packed)) KernelArgs {
   int64_t kernel_base;
   int64_t kernel_size;
   int64_t heap_base;
   int64_t heap_size;
   int64_t env_base;
   int64_t env_size;
} KernelArgs;

// Our kernel entry point takes a pointer to a KernelArgs struct, which passes the size parameters
// of our kernel and the heap.
void kernel_start(KernelArgs* args_ptr) {
   extern void kernel_main();

   kernel_main();

   return;
}

