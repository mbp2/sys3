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

pub static COM2: Spinlock<SerialPort<Pio<u8>>> = Spinlock::new(SerialPort::<Pio<u8>>::new(0x3F8));

/// An x86 IO port-mapped UART implementation.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub struct SerialPort<T: HardwareIo> {
   /// Data register; need to receive, read, and write.
   pub data: T,

   /// Interrupt enable port.
   ///
   /// Write only.
   pub interrupt_enable: T,

   /// FIFO-control port.
   ///
   /// Write only.
   pub fifo_control: T,

   /// Line control port.
   ///
   /// Write only.
   pub line_control: T,

   /// Modem control port.
   ///
   /// Write only.
   pub modem_control: T,

   /// Line status port.
   ///
   /// Read only.
   pub line_status: ReadOnly<T>,
   pub modem_status: ReadOnly<T>,
}

impl SerialPort<Pio<u8>> {
   /// Creates a new serial port interface on the given I/O base port.
   ///
   /// This function is unsafe because the caller must ensure that the given base address
   /// really points to a serial port device and that the caller has the necessary rights
   /// to perform the I/O operation.
   pub const fn new(base: u16) -> Self {
      return SerialPort{
         data: Pio::new(base),
         interrupt_enable: Pio::new(base + 1),
         fifo_control: Pio::new(base + 2),
         line_control: Pio::new(base + 3),
         modem_control: Pio::new(base + 4),
         line_status: ReadOnly::new(Pio::new(base + 5)),
         modem_status: ReadOnly::new(Pio::new(base + 6)),
      };
   }
}

impl<T: HardwareIo> SerialPort<T>
   where
      T::Value: From<u8> + TryInto<u8>,
{
   /// Initializes the serial port.
   ///
   /// The default configuration of [38400/8-N-1](https://en.wikipedia.org/wiki/8-N-1) is used.
   pub fn initialise(&mut self) {
      self.interrupt_enable.write(0x00.into());
      self.line_control.write(0x80.into());
      self.data.write(0x01.into());
      self.interrupt_enable.write(0x00.into());
      self.line_control.write(0x03.into());
      self.fifo_control.write(0xC7.into());
      self.modem_control.write(0x0B.into());
      self.interrupt_enable.write(0x01.into());
   }

   /// Sends a byte of data through the serial port.
   ///
   /// # Arguments
   ///
   /// * `byte` - The data byte to send.
   pub fn send(&mut self, byte: u8) {
      while !self.line_status_flags().contains(LineStatusFlags::OUTPUT_EMPTY){}
      self.data.write(byte.into());
   }

   /// Receives a byte on the serial port.
   pub fn receive(&mut self) -> Option<u8> {
      return if self.line_status_flags().contains(LineStatusFlags::INPUT_FULL) {
         Some(
            (self.data.read() & 0xFF.into())
               .try_into()
               .unwrap_or(0)
         )
      } else {
         None
      };
   }

   /// Writes a byte of data to the serial port.
   ///
   /// # Arguments
   ///
   /// * `byte` - The byte of data to write.
   pub fn write(&mut self, byte: u8) {
      match byte {
         8 | 0x7F => {
            self.send(8);
            self.send(b' ');
            self.send(8);
         }

         b'\n' => {
            self.send(b'\r');
            self.send(b'\n');
         }

         _ => {
            self.send(byte);
         }
      }
   }

   fn line_status_flags(&self) -> LineStatusFlags {
      return LineStatusFlags::from_bits_truncate(
         (self.line_status.read() & 0xFF.into())
            .try_into()
            .unwrap_or(0)
      );
   }
}

impl fmt::Write for SerialPort<Pio<u8>> {
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
   crate::{
      io::{HardwareIo, ReadOnly},
      syscall::pio::Pio,
   },
   bitflags::bitflags,
   core::{
      fmt,
      sync::atomic::{
         AtomicPtr, Ordering,
      },
   },
   spinning_top::Spinlock,
};
