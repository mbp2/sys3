extern void kernel_start(void) {
   extern {
      void kernel_main(void);
   }

   kernel_main();

   return;
}
