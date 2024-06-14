use crate::Terminal;
use std::{
    io::{self, Read},
    time::{Duration, Instant},
};

impl Terminal {
    pub fn clear_input(&mut self) -> io::Result<()> {
        self.pollkey(Some(Duration::ZERO))?;
        Ok(self.kbd_buf.clear())
    }

    pub fn wait_enter(&mut self, timeout: Option<Duration>) -> io::Result<KeyEvent> {
        self.clear_input()?;

        let end_time = timeout.map(|t| Instant::now() + t);
        loop {
            if self.pollkey(end_time.map(|end| (end - Instant::now())))? {
                break Ok(KeyEvent::Timeout);
            }

            match self.kbd_buf.pop() {
                Some(Key::Enter) => return Ok(KeyEvent::Key(Key::Enter)),
                _ => (),
            };
        }
    }

    pub fn get_key(&mut self, want_key: impl Fn(Key) -> bool) -> io::Result<Option<Key>> {
        self.get_key_timeout(Some(Duration::ZERO), want_key)
    }

    pub fn get_key_timeout(
        &mut self,
        timeout: Option<Duration>,
        want_key: impl Fn(Key) -> bool,
    ) -> io::Result<Option<Key>> {
        self.pollkey(timeout)?;
        loop {
            match self.kbd_buf.pop() {
                Some(k) if want_key(k) => return Ok(Some(k)),
                Some(_) => (),
                None => return Ok(None),
            }
        }
    }

    fn pollkey(&mut self, timeout: Option<Duration>) -> io::Result<bool> {
        if !oca_io::poll::poll_read_fd(&self.file, timeout) {
            // no data received
            return Ok(true);
        }

        let mut buf = [0u8; 64];
        let n = self.file.read(&mut buf)?;
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
            self.kbd_buf.push(match b {
                b'\n' => Key::Enter,
                0x7F => Key::Back,
                0x1B => match next() {
                    Some(b'[') => match next() {
                        Some(b'A') => Key::Up,
                        Some(b'B') => Key::Down,
                        Some(b'C') => Key::Right,
                        Some(b'D') => Key::Left,
                        _ => Key::Unknown,
                    },
                    None => Key::Esc,
                    _ => Key::Unknown,
                },
                ch @ (b'A'..=b'Z' | b'a'..=b'z') => Key::Char(ch.to_ascii_lowercase()),
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
    Enter,
    Back,
    Esc,
    Char(u8),

    Up,
    Down,
    Right,
    Left,

    Unknown,
}
