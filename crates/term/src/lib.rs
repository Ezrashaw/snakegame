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
    process, thread,
};

pub struct Terminal {
    file: File,
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

        // SAFETY: we can always wrap FD 0 (stdin).
        let mut file = unsafe { File::from_raw_fd(0) };
        write!(file, "\x1B[?1049h\x1B[?25l\x1B[2J\x1B[H")?;

        Ok(Self {
            file,
            kbd_buf: CircularBuffer::new(),
            old_termios,
            term_size: oca_io::get_termsize(),
        })
    }

    #[must_use]
    pub const fn size(&self) -> (u16, u16) {
        self.term_size
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
        termios::restore(self.old_termios);
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.close()
    }
}
