use std::{
    io::{self, ErrorKind, Read},
    ptr,
};

use super::Terminal;

impl Terminal {
    pub fn poll_key(&mut self, timeout_ms: u64) -> io::Result<Option<Key>> {
        let mut poll_fd = libc::pollfd {
            fd: libc::STDIN_FILENO,
            events: libc::POLLIN,
            revents: 0,
        };

        let time_spec = libc::timespec {
            tv_sec: (timeout_ms / 1000) as i64,
            tv_nsec: ((timeout_ms % 1000) * 1_000_000) as i64,
        };

        let res = unsafe {
            libc::ppoll(
                ptr::from_mut(&mut poll_fd),
                1,
                ptr::from_ref(&time_spec),
                ptr::null::<libc::sigset_t>(),
            )
        };

        match res {
            -1 => panic!("libc call failed"),
            0 => Ok(None),
            1 => {
                assert!(poll_fd.revents == libc::POLLIN);
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

    #[allow(unused)]
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
            ch @ (b'A'..=b'Z' | b'a'..=b'z') => Key::Char(ch),
            x => {
                // println!("\x1B[H{x}                 ");
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

pub(super) fn set_non_block(mut flags: i32, non_block: bool) -> i32 {
    if non_block {
        flags |= libc::O_NONBLOCK;
    } else {
        flags &= !libc::O_NONBLOCK;
    }

    let res = unsafe { libc::fcntl(libc::STDIN_FILENO, libc::F_SETFL, flags) };
    assert!(res != -1);

    flags
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Key {
    CrtlC,
    Enter,
    Char(u8),

    Up,
    Down,
    Right,
    Left,

    Unknown(u8, u8),
}
