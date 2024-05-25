#![feature(strict_overflow_ops, associated_type_defaults, let_chains)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

mod ansi;
mod draw;
mod stdin;
mod stdout;
mod termios;

#[cfg(not(all(target_os = "linux")))]
compile_error!("This program only runs on Linux");

pub use ansi::from_pansi;
pub use draw::{draw, draw_centered, Box, Draw, DrawCtx};
pub use stdin::{Key, KeyEvent};
pub use stdout::{ansi_str_len, Color, Rect};

use std::{
    fs::File,
    io::{self, Write},
    os::fd::FromRawFd,
    ptr, thread,
};

use self::termios::Termios;

// remember that coordinates begin at one, not zero.

pub struct Terminal {
    out: File,
    in_: File,

    old_termios: Termios,

    #[allow(unused)]
    stdin_flags: i32,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        let old_termios = termios::init(|t| {
            t.set_sig(false);
            t.set_canonical(false);
            t.set_echo(false);
        });

        // set stdin to non-blocking mode using fcntl.
        let stdin_flags = unsafe { libc::fcntl(libc::STDIN_FILENO, libc::F_GETFL) };
        assert!(stdin_flags != -1);
        let stdin_flags = stdin::set_non_block(stdin_flags, true);

        // SAFETY: we can always wrap FD 1 (stdout).
        let mut out = unsafe { File::from_raw_fd(1) };
        // SAFETY: we can always wrap FD 0 (stdin).
        let in_ = unsafe { File::from_raw_fd(0) };

        write!(out, "\x1B[?1049h\x1B[?25l")?;
        write!(out, "\x1B[2J\x1B[H")?;

        Ok(Self {
            out,
            in_,
            old_termios,
            stdin_flags,
        })
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

#[must_use]
pub fn get_termsize() -> (u16, u16) {
    let mut win_size = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let res = unsafe {
        libc::ioctl(
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ,
            ptr::from_mut(&mut win_size),
        )
    };
    assert_eq!(res, 0);
    (win_size.ws_col, win_size.ws_row)
}
