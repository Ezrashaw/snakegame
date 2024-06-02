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
    termios::{self, Termios},
    CircularBuffer,
};

use std::{
    fs::File,
    io::{self, Write},
    os::fd::FromRawFd,
    thread,
};

pub struct Terminal {
    out: File,
    in_: File,
    kbd_buf: CircularBuffer<Key, 64>,

    old_termios: Termios,
    term_size: (u16, u16),
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        let old_termios = termios::init(|t| {
            t.set_sig(false);
            t.set_canonical(false);
            t.set_echo(false);
            t.set_ixon(false);
        });

        // SAFETY: we can always wrap FD 1 (stdout).
        let mut out = unsafe { File::from_raw_fd(1) };
        write!(out, "\x1B[?1049h\x1B[?25l\x1B[2J\x1B[H")?;

        // SAFETY: we can always wrap FD 0 (stdin).
        let in_ = unsafe { File::from_raw_fd(0) };

        Ok(Self {
            out,
            in_,
            kbd_buf: CircularBuffer::new(),
            old_termios,
            term_size: oca_io::get_termsize(),
        })
    }

    #[must_use]
    pub const fn size(&self) -> (u16, u16) {
        self.term_size
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Don't clear terminal if panicking so that we can see the error message.
        if !thread::panicking() {
            write!(&mut self.out, "\x1B[2J\x1B[H\x1B[?1049l").unwrap();
        }
        write!(&mut self.out, "\x1B[?25h").unwrap();
        termios::restore(self.old_termios);
    }
}
