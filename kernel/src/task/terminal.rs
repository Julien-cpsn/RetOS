use crate::printer::buffer::WRITER;
use crate::println;
use crate::terminal::args::CliArgs;
use crate::terminal::cli::{handle_command, set_max_verbosity, Cli};
use alloc::format;
use core::pin::Pin;
use core::task::{Context, Poll};
use crossbeam_queue::ArrayQueue;
use futures_util::task::AtomicWaker;
use futures_util::{Stream, StreamExt};
use goolog::set_target;
use no_std_clap_core::error::ParseError;
use no_std_clap_core::parser::Parser;
use pc_keyboard::{DecodedKey, KeyCode};
use spin::Once;
use yansi::Paint;

static SCANCODE_QUEUE: Once<ArrayQueue<DecodedKey>> = Once::new();
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
    type Item = DecodedKey;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<DecodedKey>> {
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

pub fn add_key(key: DecodedKey) {
    if let Some(queue) = SCANCODE_QUEUE.get() {
        match queue.push(key) {
            Ok(_) => WAKER.wake(),
            Err(_) => println!("WARNING: scancode queue full; dropping keyboard input")
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub async fn handle_keyboard() {
    set_target!("Keyboard");

    let mut scancodes = ScancodeStream::new();

    println!("======= User input starts =======");

    let mut cli = Cli::new(
        format!("{}{} ", "RetOS".dim(), '$'.white()),
        7,
        WRITER.clone()
    );

    cli.reset_line();

    while let Some(key) = scancodes.next().await {
        match key {
            DecodedKey::RawKey(key) => match key {
                KeyCode::Escape => {}
                KeyCode::Backspace => {}
                KeyCode::Tab => {}
                KeyCode::Delete => {}
                KeyCode::End => {}
                KeyCode::Return => {}
                KeyCode::ArrowUp => cli.previous_command(),
                KeyCode::ArrowDown => cli.next_command(),
                KeyCode::ArrowLeft => cli.move_cursor_left(),
                KeyCode::ArrowRight => cli.move_cursor_right(),
                _ => {}
            },
            DecodedKey::Unicode(char) => {
                if let Some(command) = cli.handle_scancode(char as u8) {
                    match CliArgs::parse_str(&command) {
                        Ok(cli_args) => {
                            set_max_verbosity(cli_args.verbose);
                            handle_command(cli_args.command);
                        },
                        Err(parse_error) => {
                            match parse_error {
                                ParseError::EmptyInput => {}
                                ParseError::Help(help) => println!("{}", help),
                                ParseError::MissingArgument(argument) => println!("{} {}", "Missing required argument:".red(), argument),
                                ParseError::InvalidValue(value) => println!("{} {}", "Invalid value:".red(), value),
                                ParseError::UnknownArgument(argument) => println!("{} {}", "Unknown argument:".red(), argument),
                                ParseError::UnknownSubcommand => println!("{} {}", "Unknown command:".red(), command),
                                ParseError::InvalidFormat(format) => println!("{} {}", "Invalid format:".red(), format),
                                ParseError::UnknownEnumVariant(value, possible_values) => println!("{} {}, possible values are: {}", "Invalid value:".red(), value, possible_values)
                            }
                        }
                    }

                    cli.reset_line();
                }
            },
        }
    }
}