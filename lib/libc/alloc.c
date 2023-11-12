#include <alloc.h>
#include <stdmem.h>

#define MAX_ALLOC_ALLOWED 20

size_t _heap_size;

static uint32_t AllocatedNumber = 0;
static uint32_t HeapBaseAddr = 0x0;
static unsigned char memory_stat[_heap_size];
AllocInfo metadata_info[MAX_ALLOC_ALLOWED] = {0};

void* malloc(size_t size) {
   int j = 0;
   int index = 0;
   int initial_gap = 0;
   int gap = 0;
   int heap_index = 0;

   void* addr = NULL;

   bool flag = false;
   bool initial_flag = false;

   AllocInfo temp_info = {0};

   if (AllocatedNumber >= MAX_ALLOC_ALLOWED) {
      return NULL;
   }

   for (index = 0; index < AllocatedNumber; index++) {
      if (metadata_info[index+1].address != 0){
         initial_gap = metadata_info[0].address - HeapBaseAddr;

         if (initial_gap >= size) {
            initial_flag = true;
            break;
         } else {
            gap = metadata_info[index + 1].address - (metadata_info[index].address + metadata_info[index].size);

            if (gap >= size) {
               flag = true;
               break;
            }
         }
      }
   }

   if (flag == true) { /* Get Index for allocating memory for case two. */
      heap_index = ((metadata_info[index].address + metadata_info[index].size) - HeapBaseAddr);

      for (j = MAX_ALLOC_ALLOWED - 1; j > index + 1; j--) {
         memcpy(&metadata_info[j], &metadata_info[j - 1], sizeof(AllocInfo));
      }
   } else if (initial_flag == true) { /* Get index for allocating memory for case three. */
      heap_index = 0;

      for (j = MAX_ALLOC_ALLOWED - 1; j > index + 1; j--) {
         memcpy(&metadata_info[j], &metadata_info[j - 1], sizeof(AllocInfo));
      }
   } else { /* Get Index for allocating memory for case one. */
      if (AllocatedNumber != 0) {
         heap_index = ((metadata_info[index - 1].address - metadata_info[index - 1].size) - HeapBaseAddr);
      } else {
         heap_index = 0;
      }
   }

   addr = &memory_stat[heap_index];
   metadata_info[index].address = HeapBaseAddr + heap_index;
   metadata_info[index].size = size;

   return addr;
}
