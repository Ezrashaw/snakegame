mod stdin;
mod stdout;
mod syscall;
mod termios;

pub use stdin::Key;
pub use stdout::{Color, Rect};

use std::{
    fs::File,
    io::{self, Write},
    os::fd::FromRawFd,
    ptr,
};

use crate::terminal::{
    stdin::F_GETFL,
    syscall::{syscall2, syscall3, SYS_fcntl, SYS_ioctl},
    termios::{STDIN_FD, STDOUT_FD},
};

use self::termios::Termios;

// remember that coordinates begin at one, not zero.

pub struct Terminal {
    out: File,
    in_: File,

    old_termios: Termios,
    flags: i64,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        let old_termios = termios::init(|t| {
            t.set_sig(false);
            t.set_canonical(false);
            t.set_echo(false);
        });

        // set stdin to non-blocking mode using fcntl.
        let flags = unsafe { syscall2(SYS_fcntl, STDIN_FD, F_GETFL) } as i64;
        assert!(flags != -1);
        let flags = stdin::set_non_block(flags, true);

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
            flags,
        })
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        write!(&mut self.out, "\x1B[2J\x1B[H").unwrap();
        write!(&mut self.out, "\x1B[?1049l\x1B[?25h").unwrap();
        termios::restore(self.old_termios);
    }
}

pub fn get_termsize() -> (u16, u16) {
    #[derive(Default)]
    #[repr(C)]
    struct WinSize {
        ws_row: u16,
        ws_col: u16,
        ws_xpixel: u16, /* unused */
        ws_ypixel: u16, /* unused */
    }

    let mut win_size = WinSize::default();
    let res = unsafe {
        syscall3(
            SYS_ioctl,
            STDOUT_FD,
            0x5413, // TIOCGWINSZ
            ptr::from_mut(&mut win_size) as u64,
        )
    };
    assert_eq!(res, 0);
    (win_size.ws_col, win_size.ws_row)
}
