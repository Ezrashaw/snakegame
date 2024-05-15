use std::{
    io::{self, ErrorKind, Read},
    ptr,
};

use crate::terminal::{
    syscall::{syscall3, SYS_fcntl, SYS_poll},
    termios::STDIN_FD,
};

use super::Terminal;

// fcntl values
pub(super) const F_GETFL: u64 = 3;
pub(super) const F_SETFL: u64 = 4;
pub(super) const O_NONBLOCK: i64 = 2048;

impl Terminal {
    pub fn poll_key(&mut self, timeout_ms: u64) -> io::Result<Option<Key>> {
        const POLLIN: u16 = 0x1;

        #[repr(C)]
        struct PollFD {
            fd: i32,      /* file descriptor */
            events: u16,  /* requested events */
            revents: u16, /* returned events */
        }

        let mut poll_fd = PollFD {
            fd: 0,
            events: POLLIN,
            revents: 0,
        };
        let res =
            unsafe { syscall3(SYS_poll, ptr::from_mut(&mut poll_fd) as u64, 1, timeout_ms) } as i64;

        match res {
            -1 => panic!("syscall failed"),
            0 => Ok(None),
            1 => {
                assert!(poll_fd.revents == POLLIN);
                Ok(self.get_last_key()?)
            }
            _ => unreachable!(),
        }
    }

    pub fn wait_key(&mut self, key: Key) -> io::Result<()> {
        // flush any keys currently in the buffer (we don't want prompts using
        // this function to immediately close)
        let _ = self.get_last_key()?;

        self.flags = set_non_block(self.flags, false);
        loop {
            if self.get_key()? == key {
                break;
            }
        }
        self.flags = set_non_block(self.flags, true);
        Ok(())
    }

    pub fn get_key_blocking(&mut self) -> io::Result<Key> {
        self.flags = set_non_block(self.flags, false);
        let key = self.get_key()?;
        self.flags = set_non_block(self.flags, true);
        Ok(key)
    }

    pub fn get_last_key(&mut self) -> io::Result<Option<Key>> {
        let mut last_key = None;
        loop {
            match self.get_key() {
                Ok(key) => last_key = Some(key),
                Err(err) if matches!(err.kind(), ErrorKind::WouldBlock) => return Ok(last_key),
                Err(err) => return Err(err),
            }
        }
    }

    pub fn get_key(&mut self) -> io::Result<Key> {
        Ok(match self.readbyte()? {
            0x3 => Key::CrtlC,
            b'\n' => Key::Enter,
            0x1B => {
                if self.readbyte()? != b'[' {
                    unreachable!(); // ANSI code formatted wrong; missing '[' after ESC
                }
                match self.readbyte()? {
                    b'A' => Key::Up,
                    b'B' => Key::Down,
                    b'C' => Key::Right,
                    b'D' => Key::Left,
                    x => Key::Unknown(0x1B, x),
                }
            }
            x => {
                // println!("{x}");
                Key::Unknown(x, 0x0)
            }
        })
    }

    fn readbyte(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.in_.read_exact(&mut buf)?;
        let [byte] = buf;
        Ok(byte)
    }
}

pub(super) fn set_non_block(mut flags: i64, non_block: bool) -> i64 {
    if non_block {
        flags |= O_NONBLOCK;
    } else {
        flags &= !O_NONBLOCK;
    }

    let res = unsafe { syscall3(SYS_fcntl, STDIN_FD, F_SETFL, flags as u64) } as i64;
    assert!(res != -1);

    flags
}

#[derive(Debug, PartialEq, Eq)]
pub enum Key {
    CrtlC,
    Enter,

    Up,
    Down,
    Right,
    Left,

    Unknown(u8, u8),
}
