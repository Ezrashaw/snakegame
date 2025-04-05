use crate::Terminal;
use core::time::Duration;
use oca_io::{Result, timer::Instant};

impl Terminal {
    pub fn clear_input(&mut self) -> Result<()> {
        self.pollkey(Some(Duration::ZERO))?;
        self.kbd_buf.clear();
        Ok(())
    }

    pub fn wait_enter(&mut self, timeout: Option<Duration>) -> Result<KeyEvent> {
        self.wait_key(|k| k == Key::Enter, timeout)
    }

    pub fn wait_key(
        &mut self,
        want_key: impl Fn(Key) -> bool,
        timeout: Option<Duration>,
    ) -> Result<KeyEvent> {
        self.clear_input()?;

        let end_time = timeout.map(|t| Instant::now().unwrap() + t);
        loop {
            if self.pollkey(end_time.map(|end| (end - Instant::now().unwrap())))? {
                break Ok(KeyEvent::Timeout);
            }

            loop {
                match self.kbd_buf.pop() {
                    Some(k) if want_key(k) => return Ok(KeyEvent::Key(k)),
                    Some(_) => (),
                    None => break,
                }
            }
        }
    }

    pub fn get_key(&mut self, want_key: impl Fn(Key) -> bool) -> Result<Option<Key>> {
        self.get_key_timeout(Some(Duration::ZERO), want_key)
    }

    pub const fn key_iter(&mut self) -> KeyIter<'_> {
        KeyIter(self)
    }

    pub fn get_key_timeout(
        &mut self,
        timeout: Option<Duration>,
        want_key: impl Fn(Key) -> bool,
    ) -> Result<Option<Key>> {
        self.pollkey(timeout)?;
        loop {
            match self.kbd_buf.pop() {
                Some(k) if want_key(k) => return Ok(Some(k)),
                Some(_) => (),
                None => return Ok(None),
            }
        }
    }

    fn pollkey(&mut self, timeout: Option<Duration>) -> Result<bool> {
        if !oca_io::poll::poll_read_fd(&self.file, timeout)? {
            // no data received
            return Ok(true);
        }

        let mut buf = [0u8; 64];
        let n = self.file.read(&mut buf)?;
        assert!(n != 64);

        let mut i = 0;
        let mut next = || {
            #[allow(clippy::cast_possible_truncation)]
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
                ch @ (b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9') => {
                    Key::Char(ch.to_ascii_lowercase())
                }
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

pub struct KeyIter<'a>(&'a mut Terminal);

impl Iterator for KeyIter<'_> {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pollkey(Some(Duration::ZERO)).ok()?;
        self.0.kbd_buf.pop()
    }
}
