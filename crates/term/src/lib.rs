#![feature(strict_overflow_ops, associated_type_defaults, if_let_guard)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]

mod ansi;
mod draw;
mod stdin;
mod stdout;

pub use ansi::{ansi_str_len, from_pansi};
use oca_io::{
    termios::{self, Termios},
    CircularBuffer,
};

pub use draw::{draw, draw_centered, update, Box, CenteredStr, Draw, DrawCtx, Pixel, Popup};
pub use stdin::{Key, KeyEvent};
pub use stdout::{Color, Rect};

use std::{
    fs::File,
    io::{self, Write},
    os::fd::FromRawFd,
    thread,
};

// remember that coordinates begin at one, not zero.

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
        });

        // SAFETY: we can always wrap FD 1 (stdout).
        let mut out = unsafe { File::from_raw_fd(1) };
        // SAFETY: we can always wrap FD 0 (stdin).
        let in_ = unsafe { File::from_raw_fd(0) };

        write!(out, "\x1B[?1049h\x1B[?25l")?;
        write!(out, "\x1B[2J\x1B[H")?;

        let term_size = oca_io::get_termsize();

        #[cfg(feature = "term_debug")]
        draw_debug_lines(term_size.0, term_size.1);

        Ok(Self {
            out,
            in_,
            kbd_buf: CircularBuffer::new(),
            old_termios,
            term_size,
        })
    }

    #[must_use]
    pub const fn size(&self) -> (u16, u16) {
        self.term_size
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Don't reset terminal if panicking so that we can see the error message.
        if !thread::panicking() {
            write!(&mut self.out, "\x1B[2J\x1B[H").unwrap();
            write!(&mut self.out, "\x1B[?1049l\x1B[?25h").unwrap();
            termios::restore(self.old_termios);
        }
    }
}

#[cfg(feature = "term_debug")]
fn draw_debug_lines(w: u16, h: u16) {
    for i in (1..h).step_by(2) {
        print!("\x1B[{i};0H{i:0>2}--+");
        for x in (1..(w - 15)).step_by(5) {
            if x % 10 == 1 {
                print!("--{:0>2}+", x + 9);
            } else {
                print!("----+");
            }
        }
        println!("\x1B[{}G{i:0>2}", w - 1);
    }
}
