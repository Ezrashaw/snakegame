#![feature(strict_overflow_ops, associated_type_defaults)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]

mod draw;
mod stdin;
mod stdout;

pub use draw::{draw, draw_centered, update, Box, CenteredStr, Draw, DrawCtx, Pixel, Popup};
pub use stdin::{Key, KeyEvent};
pub use stdout::{ansi_str_len, Color, Rect};

use oca_io::{
    signal::{Signal, SignalFile},
    termios::{self, Termios},
    CircularBuffer,
};

use std::{
    fs::File,
    io::{self, Write},
    os::fd::FromRawFd,
    process, thread,
    time::Duration,
};

pub struct Terminal {
    file: File,
    kbd_buf: CircularBuffer<Key, 64>,
    signalfd: SignalFile,

    old_termios: Termios,
    term_size: (u16, u16),
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        let old_termios = termios::init(|t| {
            t.set_canonical(false);
            t.set_echo(false);
            t.set_ixon(false);
        });

        // SAFETY: we can always wrap FD 0 (stdin).
        let mut file = unsafe { File::from_raw_fd(0) };
        write!(file, "\x1B[?1049h\x1B[?25l\x1B[2J\x1B[H")?;

        Ok(Self {
            file,
            kbd_buf: CircularBuffer::new(),
            signalfd: SignalFile::new(&[
                Signal::Interrupt,
                Signal::Terminate,
                Signal::WindowChange,
            ]),
            old_termios,
            term_size: oca_io::get_termsize(),
        })
    }

    #[must_use]
    pub const fn size(&self) -> (u16, u16) {
        self.term_size
    }

    pub fn process_signals(&mut self) -> io::Result<bool> {
        if oca_io::poll::poll_read_fd(&self.signalfd, Some(Duration::ZERO)) {
            match self.signalfd.get_signal()? {
                Signal::Interrupt | Signal::Terminate => return Ok(true),
                Signal::WindowChange => self.exit_with_error(
                    "detected that the terminal size changed; this is not supported",
                ),
            }
        }
        Ok(false)
    }

    pub fn exit_with_error(&mut self, msg: impl AsRef<str>) -> ! {
        self.close();
        eprintln!("\x1B[1;31merror\x1B[0m: {}", msg.as_ref());
        process::exit(1)
    }

    fn close(&mut self) {
        // Don't clear terminal if panicking so that we can see the error message.
        if !thread::panicking() {
            write!(&mut self.file, "\x1B[2J\x1B[H\x1B[?1049l").unwrap();
        }
        write!(&mut self.file, "\x1B[?25h").unwrap();
        self.old_termios.sys_set();
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.close();
    }
}
