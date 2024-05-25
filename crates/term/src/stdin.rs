use std::{
    io::{self, ErrorKind, Read},
    ptr,
    time::{Duration, Instant},
};

use super::Terminal;

impl Terminal {
    pub fn wait_key(
        &mut self,
        want_key: impl Fn(Key) -> bool,
        timeout: Option<Duration>,
        only_new: bool,
    ) -> io::Result<KeyEvent> {
        if only_new && self.get_last_key()? == Some(Key::CrtlC) {
            return Ok(KeyEvent::Exit);
        }

        let end_time = timeout.map(|t| Instant::now() + t);
        loop {
            if !poll_key(end_time.map(|end| (end - Instant::now()).as_millis() as u64)) {
                return Ok(KeyEvent::Timeout);
            }

            match self.read_key()? {
                Key::CrtlC => return Ok(KeyEvent::Exit),
                k if want_key(k) => return Ok(KeyEvent::Key(k)),
                _ => (),
            };
        }
    }

    fn get_last_key(&mut self) -> io::Result<Option<Key>> {
        let mut last_key = None;
        loop {
            match self.read_key() {
                Ok(Key::CrtlC) => return Ok(Some(Key::CrtlC)),
                Ok(key) => last_key = Some(key),
                Err(err) if matches!(err.kind(), ErrorKind::WouldBlock) => return Ok(last_key),
                Err(err) => return Err(err),
            }
        }
    }

    fn read_key(&mut self) -> io::Result<Key> {
        Ok(match self.readbyte()? {
            0x3 => Key::CrtlC,
            b'\n' => Key::Enter,
            0x1B => {
                let openbracket = self.readbyte();
                match openbracket {
                    Ok(b'[') => (),
                    Ok(val) => return Ok(Key::Unknown(0x1B, val)),
                    Err(err) if matches!(err.kind(), ErrorKind::WouldBlock) => {
                        return Ok(Key::Unknown(0x1B, 0xFF))
                    }
                    Err(err) => return Err(err),
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

pub fn set_non_block(mut flags: i32, non_block: bool) -> i32 {
    if non_block {
        flags |= libc::O_NONBLOCK;
    } else {
        flags &= !libc::O_NONBLOCK;
    }

    let res = unsafe { libc::fcntl(libc::STDIN_FILENO, libc::F_SETFL, flags) };
    assert!(res != -1);

    flags
}

fn poll_key(timeout_ms: Option<u64>) -> bool {
    let mut poll_fd = libc::pollfd {
        fd: libc::STDIN_FILENO,
        events: libc::POLLIN,
        revents: 0,
    };

    let time_spec = timeout_ms.map(|t_ms| libc::timespec {
        tv_sec: (t_ms / 1000).try_into().unwrap(),
        tv_nsec: ((t_ms % 1000) * 1_000_000).try_into().unwrap(),
    });

    let res = unsafe {
        libc::ppoll(
            ptr::from_mut(&mut poll_fd),
            1,
            // VERY IMPORTANT: take the reference with `as_ref`, not in a closure with
            // ptr::from_ref because the reference's (represented as a raw pointer) lifetime is
            // bound to the closure, not the libc call. Otherwise this is UB... oops. This was okay
            // in debug mode, but release mode optimized it into UB.
            time_spec.as_ref().map_or(ptr::null(), ptr::from_ref),
            ptr::null::<libc::sigset_t>(),
        )
    };

    match res {
        -1 => panic!("libc call failed"),
        0 => false,
        1 => {
            assert!(poll_fd.revents == libc::POLLIN);
            true
        }
        _ => unreachable!(),
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyEvent {
    Timeout,
    Exit,
    Key(Key),
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
