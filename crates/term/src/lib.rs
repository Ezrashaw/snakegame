#![feature(strict_overflow_ops, associated_type_defaults, iter_repeat_n)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]
#![cfg_attr(not(feature = "std"), no_std)]

mod draw;
mod stdin;
mod stdout;

pub use draw::{draw, draw_centered, update, Box, CenteredStr, Clear, Draw, DrawCtx, Pixel, Popup};
pub use stdin::{Key, KeyEvent};
pub use stdout::{ansi_str_len, Color, Direction, Rect};

use oca_io::{
    file::File,
    signal::{Signal, SignalFile},
    termios::{self, Termios},
    CircularBuffer, Result, StaticString,
};

use core::{fmt::Write, time::Duration};

pub struct Terminal {
    file: File,
    out_buf: StaticString<4096>,
    kbd_buf: CircularBuffer<Key, 64>,
    signalfd: SignalFile,

    old_termios: Termios,
    cursor: Option<(u16, u16)>,
    term_size: (u16, u16),
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let old_termios = termios::init(|t| {
            t.set_canonical(false);
            t.set_echo(false);
            t.set_ixon(false);
        })?;

        let mut file = File::stdin().unwrap();
        write!(file, "\x1B[?1049h\x1B[2J\x1B[H")?;
        Self::set_cursor_vis(&mut file, false)?;

        Ok(Self {
            file,
            out_buf: StaticString::new(),
            kbd_buf: CircularBuffer::new(),
            signalfd: SignalFile::new(&[
                Signal::Interrupt,
                Signal::Terminate,
                Signal::WindowChange,
            ])?,
            old_termios,
            cursor: None,
            term_size: oca_io::get_termsize()?,
        })
    }

    #[must_use]
    pub const fn size(&self) -> (u16, u16) {
        self.term_size
    }

    pub fn process_signals(&mut self) -> Result<bool> {
        if oca_io::poll::poll_read_fd(self.signalfd.as_file(), Some(Duration::ZERO))? {
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
        write!(File::from_fd(2), "\x1B[1;31merror\x1B[0m: {}", msg.as_ref()).unwrap();
        oca_io::exit(1)
    }

    fn close(&mut self) {
        #[cfg(feature = "std")]
        {
            // Don't clear terminal if panicking so that we can see the error message.
            if !std::thread::panicking() {
                write!(&mut self.file, "\x1B[2J\x1B[H\x1B[?1049l").unwrap();
            }
        }
        #[cfg(not(feature = "std"))]
        {
            write!(&mut self.file, "\x1B[2J\x1B[H\x1B[?1049l").unwrap();
        }

        Self::set_cursor_vis(&mut self.file, true).unwrap();
        self.old_termios.sys_set().unwrap();
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.close();
    }
}
