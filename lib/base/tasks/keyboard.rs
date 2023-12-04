pub static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
pub static KEYBOARD_WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub fn add_scancode(scancode: u8) {
   if let Ok(queue) = SCANCODE_QUEUE.try_get() {
      if let Err(_) = queue.push(scancode) {
         log::warn!("scancode queue full; dropping keyboard input");
      } else {
         KEYBOARD_WAKER.wake();
      }
   } else {
      log::warn!("scancode queue uninitialised");
   }
}

pub struct ScancodeStream {
   _private: (),
}

impl ScancodeStream {
   pub fn new() -> Self {
      SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
         .expect("ScancodeStream initializer should only be called once");
      return ScancodeStream{ _private: () };
   }
}

impl Stream for ScancodeStream {
   type Item = u8;

   fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
      let queue = SCANCODE_QUEUE
         .try_get()
         .expect("scancode queue not initialized");

      // fast path
      if let Some(scancode) = queue.pop() {
         return Poll::Ready(Some(scancode));
      }

      KEYBOARD_WAKER.register(&cx.waker());
      match queue.pop() {
         Some(scancode) => {
            KEYBOARD_WAKER.take();
            Poll::Ready(Some(scancode))
         }
         None => Poll::Pending,
      }
   }
}

pub async fn print_keypresses() {
   let mut scancodes = ScancodeStream::new();
   let mut keyboard = Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore);

   while let Some(scancode) = scancodes.next().await {
      if let Ok(Some(event)) = keyboard.add_byte(scancode) {
         if let Some(key) = keyboard.process_keyevent(event) {
            match key {
               DecodedKey::Unicode(character) => print!("{}", character),
               DecodedKey::RawKey(key) => print!("{:?}", key)
            }
         }
      }
   }
}

// IMPORTS //

use {
   crate::print,
   core::{
      pin::Pin,
      task::{Context, Poll},
   },
   conquer_once::spin::OnceCell,
   crossbeam_queue::ArrayQueue,
   futures_util::{
      stream::{Stream, StreamExt},
      task::AtomicWaker,
   },
   pc_keyboard::{
      layouts,
      DecodedKey,
      HandleControl,
      Keyboard,
      ScancodeSet1,
   },
};
