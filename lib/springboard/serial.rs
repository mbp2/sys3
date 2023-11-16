/// A thin wrapper type over a [`SerialPort`][uart16550::SerialPort]
pub struct PortWrapper(pub SerialPort);

impl PortWrapper {
   /// # Safety
   ///
   /// Unsafe because this function must be called only once.
   pub unsafe fn initialise() -> Self {
      let mut inner = unsafe { SerialPort::new(0x3F8) };
      // Initialise port
      inner.init();
      return PortWrapper(inner);
   }
}

impl fmt::Write for PortWrapper {
   fn write_str(&mut self, s: &str) -> fmt::Result {
      self.0.write_str(s)
   }
}

// IMPORTS //

use {core::fmt, uart_16550::SerialPort};
