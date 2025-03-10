use crate::interrupts::idt::KEYBOARD;
use crate::println;
use crate::terminal::cli::handle_command;
use crate::terminal::terminal::TerminalBuffer;
use core::pin::Pin;
use core::task::{Context, Poll};
use crossbeam_queue::ArrayQueue;
use embedded_cli::cli::CliBuilder;
use futures_util::task::AtomicWaker;
use futures_util::{Stream, StreamExt};
use goolog::set_target;
use pc_keyboard::{DecodedKey, KeyCode};
use spin::Once;

static SCANCODE_QUEUE: Once<ArrayQueue<u8>> = Once::new();
static WAKER: AtomicWaker = AtomicWaker::new();

pub struct ScancodeStream {
    _private: (),
}

impl Default for ScancodeStream {
    fn default() -> Self {
        Self::new()
    }
}

impl ScancodeStream {
    pub fn new() -> Self {
        // ScancodeStream::new should only be called once
        SCANCODE_QUEUE.call_once(|| ArrayQueue::new(100));
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .get()
            .expect("scancode queue not initialized");

        // fast path
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub fn add_scancode(scancode: u8) {
    if let Some(queue) = SCANCODE_QUEUE.get() {
        match queue.push(scancode) {
            Ok(_) => WAKER.wake(),
            Err(_) => println!("WARNING: scancode queue full; dropping keyboard input"),
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub async fn handle_keyboard() {
    set_target!("Keyboard");

    let mut scancodes = ScancodeStream::new();

    let mut terminal_buffer = TerminalBuffer;
    let command_buffer: [u8; 100] = [0; 100];
    let history_buffer: [u8; 100] = [0; 100];

    let mut cli = CliBuilder::default()
        .writer(&mut terminal_buffer)
        .command_buffer(command_buffer)
        .history_buffer(history_buffer)
        .prompt("> ")
        .build()
        .ok()
        .unwrap();

    while let Some(scancode) = scancodes.next().await {
        let mut keyboard = KEYBOARD.write();
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                drop(keyboard);
                match key {
                    DecodedKey::Unicode(character) => handle_command(&mut cli, character as u8),
                    DecodedKey::RawKey(key) => match key {
                        KeyCode::ArrowUp => {
                            handle_command(&mut cli, 0x1B);
                            handle_command(&mut cli, b'[');
                            handle_command(&mut cli, b'A');
                        }
                        KeyCode::ArrowDown => {
                            handle_command(&mut cli, 0x1B);
                            handle_command(&mut cli, b'[');
                            handle_command(&mut cli, b'B');
                        }
                        KeyCode::ArrowRight => {
                            handle_command(&mut cli, 0x1B);
                            handle_command(&mut cli, b'[');
                            handle_command(&mut cli, b'C');
                        }
                        KeyCode::ArrowLeft => {
                            handle_command(&mut cli, 0x1B);
                            handle_command(&mut cli, b'[');
                            handle_command(&mut cli, b'D');
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

