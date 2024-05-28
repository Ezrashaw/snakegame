use std::{
    io::{self, Read},
    time::{Duration, Instant},
};

use super::Terminal;

impl Terminal {
    pub fn clear_input(&mut self) -> bool {
        while let Some(key) = self.kbd_buf.read() {
            if matches!(key, Key::CrtlC) {
                return true;
            }
        }

        false
    }

    pub fn wait_enter(&mut self, timeout: Option<Duration>) -> io::Result<KeyEvent> {
        if self.clear_input() {
            return Ok(KeyEvent::Exit);
        }

        let end_time = timeout.map(|t| Instant::now() + t);
        loop {
            if self.pollkey(end_time.map(|end| (end - Instant::now())))? {
                break Ok(KeyEvent::Timeout);
            }

            match self.kbd_buf.read() {
                Some(Key::CrtlC) => return Ok(KeyEvent::Exit),
                Some(Key::Enter) => return Ok(KeyEvent::Key(Key::Enter)),
                _ => (),
            };
        }
    }

    pub fn get_key(&mut self, want_key: impl Fn(Key) -> bool) -> io::Result<Option<Key>> {
        self.pollkey(Some(Duration::ZERO))?;
        loop {
            match self.kbd_buf.read() {
                Some(Key::CrtlC) => return Ok(Some(Key::CrtlC)),
                Some(k) if want_key(k) => return Ok(Some(k)),
                Some(_) => (),
                None => return Ok(None),
            }
        }
    }

    fn pollkey(&mut self, timeout: Option<Duration>) -> io::Result<bool> {
        if !oca_io::poll_file(&self.in_, timeout) {
            // no data received
            return Ok(true);
        }

        let mut buf = [0u8; 64];
        let n = self.in_.read(&mut buf)?;
        assert!(n != 64);

        let mut i = 0;
        let mut next = || {
            let item = (i < n).then(|| buf[i]);
            if item.is_some() {
                i += 1;
            }
            item
        };

        while let Some(b) = next() {
            self.kbd_buf.write(match b {
                0x3 => Key::CrtlC,
                b'\n' => Key::Enter,
                0x1B => match next() {
                    Some(b'[') if let Some(n) = next() => match n {
                        b'A' => Key::Up,
                        b'B' => Key::Down,
                        b'C' => Key::Right,
                        b'D' => Key::Left,
                        _ => Key::Unknown,
                    },
                    _ => Key::Unknown,
                },
                ch @ (b'A'..=b'Z' | b'a'..=b'z') => Key::Char(ch),
                _ => {
                    // println!("\x1B[H{x}\t\t");
                    Key::Unknown
                }
            });
        }

        Ok(false)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyEvent {
    Key(Key),
    Timeout,
    Exit,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Key {
    CrtlC,
    Enter,
    Char(u8),

    Up,
    Down,
    Right,
    Left,

    Unknown,
}
