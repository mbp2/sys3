pub macro wait_for {
   ($cond:expr) => {
      while !$cond {
         core::hint::spin_loop()
      }
   }
}

bitflags!{
   /// Line status flags
   struct LineStatusFlags: u8 {
      const INPUT_FULL = 1;
      // 1 to 4 unknown
      const OUTPUT_EMPTY = 1 << 5;
      // 6 and 7 unknown
   }
}

bitflags!{
   struct InterruptEnableFlags: u8 {
      const RECEIVED = 1;
      const SENT = 1 << 1;
      const ERRORED = 1 << 2;
      const STATUS_CHANGE = 1 << 3;
      // 4 to 7 are unused
   }
}

// SERIAL PORT //

/// An x86 IO port-mapped UART implementation.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[derive(Debug)]
pub struct SerialPort(u16/*our base port*/);

impl SerialPort {
   /// The base port.
   ///
   /// Read and write.
   fn base_port(&self) -> u16 {
      self.0
   }

   /// Interrupt enable port.
   ///
   /// Write only.
   fn interrupt_enable(&self) -> u16 {
      return self.base_port() + 1;
   }

   /// FIFO-control port.
   ///
   /// Write only.
   fn fifo_control(&self) -> u16 {
      return self.base_port() + 2;
   }

   /// Line control port.
   ///
   /// Write only.
   fn line_control(&self) -> u16 {
      return self.base_port() + 3;
   }

   /// Modem control port.
   ///
   /// Write only.
   fn modem_control(&self) -> u16 {
      return self.base_port() + 4;
   }

   /// Line status port.
   ///
   /// Read only.
   fn line_status_port(&self) -> u16 {
      return self.base_port() + 5;
   }

   fn line_status(&self) -> LineStatusFlags {
      return unsafe {
         LineStatusFlags::from_bits_truncate(x86::io::inb(self.line_status_port()))
      };
   }

   /// Creates a new serial port interface on the given I/O base port.
   ///
   /// This function is unsafe because the caller must ensure that the given base address
   /// really points to a serial port device and that the caller has the necessary rights
   /// to perform the I/O operation.
   pub const unsafe fn new(base: u16) -> Self {
      return SerialPort(base);
   }

   /// Initializes the serial port.
   ///
   /// The default configuration of [38400/8-N-1](https://en.wikipedia.org/wiki/8-N-1) is used.
   pub fn init(&mut self) {
      unsafe {
         // Disable interrupts
         x86::io::outb(self.interrupt_enable(), 0x00);

         // Enable DLAB
         x86::io::outb(self.line_control(), 0x80);

         // Set maximum speed to 38400 bps by configuring DLL and DLM
         x86::io::outb(self.base_port(), 0x03);
         x86::io::outb(self.interrupt_enable(), 0x00);

         // Disable DLAB and set data word length to 8 bits
         x86::io::outb(self.line_control(), 0x03);

         // Enable FIFO, clear TX/RX queues and
         // set interrupt watermark at 14 bytes
         x86::io::outb(self.fifo_control(), 0xc7);

         // Mark data terminal ready, signal request to send
         // and enable auxiliary output #2 (used as interrupt line for CPU)
         x86::io::outb(self.modem_control(), 0x0b);

         // Enable interrupts
         x86::io::outb(self.interrupt_enable(), 0x01);
      }
   }

   /// Sends a byte on the serial port.
   pub fn send(&mut self, data: u8) {
      unsafe {
         match data {
            8 | 0x7F => {
               wait_for!(self.line_status().contains(LineStatusFlags::OUTPUT_EMPTY));
               x86::io::outb(self.base_port(), 8);
               wait_for!(self.line_status().contains(LineStatusFlags::OUTPUT_EMPTY));
               x86::io::outb(self.base_port(), b' ');
               wait_for!(self.line_status().contains(LineStatusFlags::OUTPUT_EMPTY));
               x86::io::outb(self.base_port(), 8);
            }
            _ => {
               wait_for!(self.line_status().contains(LineStatusFlags::OUTPUT_EMPTY));
               x86::io::outb(self.base_port(), data);
            }
         }
      }
   }

   /// Sends a raw byte on the serial port, intended for binary data.
   pub fn send_raw(&mut self, data: u8) {
      unsafe {
         wait_for!(self.line_status().contains(LineStatusFlags::OUTPUT_EMPTY));
         x86::io::outb(self.base_port(), data);
      }
   }

   /// Receives a byte on the serial port.
   pub fn receive(&mut self) -> u8 {
      unsafe {
         wait_for!(self.line_status().contains(LineStatusFlags::INPUT_FULL));
         x86::io::inb(self.base_port())
      }
   }
}

impl fmt::Write for SerialPort {
   fn write_str(&mut self, s: &str) -> fmt::Result {
      for byte in s.bytes() {
         self.send(byte);
      }
      Ok(())
   }
}

// MEMORY MAPPED //

#[derive(Debug)]
pub struct MmioPort {
   data: AtomicPtr<u8>,
   interrupt_enable: AtomicPtr<u8>,
   fifo_control: AtomicPtr<u8>,
   line_control: AtomicPtr<u8>,
   modem_control: AtomicPtr<u8>,
   line_status: AtomicPtr<u8>,
}

impl MmioPort {
   /// Creates a new UART interface on the given memory mapped address.
   ///
   /// This function is unsafe because the caller must ensure that the given base address
   /// really points to a serial port device.
   #[rustversion::attr(since(1.61), const)]
   pub unsafe fn new(base: usize) -> Self {
      let base_pointer = base as *mut u8;
      return MmioPort{
         data: AtomicPtr::new(base_pointer),
         interrupt_enable: AtomicPtr::new(base_pointer.add(1)),
         fifo_control: AtomicPtr::new(base_pointer.add(2)),
         line_control: AtomicPtr::new(base_pointer.add(3)),
         modem_control: AtomicPtr::new(base_pointer.add(4)),
         line_status: AtomicPtr::new(base_pointer.add(5)),
      };
   }

   /// Initializes the memory-mapped UART.
   ///
   /// The default configuration of [38400/8-N-1](https://en.wikipedia.org/wiki/8-N-1) is used.
   pub fn init(&mut self) {
      let self_interrupt_enable = self.interrupt_enable.load(Ordering::Relaxed);
      let self_line_control = self.line_control.load(Ordering::Relaxed);
      let self_data = self.data.load(Ordering::Relaxed);
      let self_fifo_control = self.fifo_control.load(Ordering::Relaxed);
      let self_modem_control = self.modem_control.load(Ordering::Relaxed);

      unsafe {
         // Disable interrupts
         self_interrupt_enable.write(0x00);

         // Enable DLAB
         self_line_control.write(0x80);

         // Set maximum speed to 38400 bps by configuring DLL and DLM
         self_data.write(0x03);
         self_interrupt_enable.write(0x00);

         // Disable DLAB and set data word length to 8 bits
         self_line_control.write(0x03);

         // Enable FIFO, clear TX/RX queues and
         // set interrupt watermark at 14 bytes
         self_fifo_control.write(0xC7);

         // Mark data terminal ready, signal request to send
         // and enable auxilliary output #2 (used as interrupt line for CPU)
         self_modem_control.write(0x0B);

         // Enable interrupts
         self_interrupt_enable.write(0x01);
      }
   }

   fn line_status(&mut self) -> LineStatusFlags {
      unsafe { LineStatusFlags::from_bits_truncate(*self.line_status.load(Ordering::Relaxed)) }
   }

   /// Sends a byte on the serial port.
   pub fn send(&mut self, data: u8) {
      let self_data = self.data.load(Ordering::Relaxed);

      unsafe {
         match data {
            8 | 0x7F => {
               wait_for!(self.line_status().contains(LineStatusFlags::OUTPUT_EMPTY));
               self_data.write(8);
               wait_for!(self.line_status().contains(LineStatusFlags::OUTPUT_EMPTY));
               self_data.write(b' ');
               wait_for!(self.line_status().contains(LineStatusFlags::OUTPUT_EMPTY));
               self_data.write(8)
            }
            _ => {
               wait_for!(self.line_status().contains(LineStatusFlags::OUTPUT_EMPTY));
               self_data.write(data);
            }
         }
      }
   }

   /// Receives a byte on the serial port.
   pub fn receive(&mut self) -> u8 {
      let self_data = self.data.load(Ordering::Relaxed);

      unsafe {
         wait_for!(self.line_status().contains(LineStatusFlags::INPUT_FULL));
         self_data.read()
      }
   }
}

impl fmt::Write for MmioPort {
   fn write_str(&mut self, s: &str) -> fmt::Result {
      for byte in s.bytes() {
         self.send(byte);
      }

      Ok(())
   }
}

// IMPORTS //

use {
   bitflags::bitflags,
   core::{
      fmt,
      sync::atomic::{
         AtomicPtr, Ordering,
      },
   },
};
