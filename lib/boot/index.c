void KernelStart(void) {
   extern {
      void KernelMain(void);
   }

   KernelMain();

   return;
}
