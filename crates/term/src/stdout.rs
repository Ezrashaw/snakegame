use crate::{Draw, Terminal};
use oca_io::Result;

impl Terminal {
    pub fn draw(&mut self, x: u16, y: u16, object: impl Draw) -> Result<()> {
        crate::draw(&mut self.out_buf, object, x, y)
    }

    pub fn update<T: Draw>(&mut self, x: u16, y: u16, object: T, u: T::Update) -> Result<()> {
        crate::update(&mut self.out_buf, object, x, y, u)
    }

    pub fn draw_centered(&mut self, object: impl Draw, rect: Rect) -> Result<(u16, u16)> {
        crate::draw_centered(&mut self.out_buf, object, rect, false)
    }

    pub fn draw_centered_hoff(
        &mut self,
        object: impl Draw,
        rect: Rect,
        hoff: bool,
    ) -> Result<(u16, u16)> {
        crate::draw_centered(&mut self.out_buf, object, rect, hoff)
    }

    pub fn flush(&mut self) -> Result<()> {
        self.file.write(self.out_buf.as_bytes())?;
        self.out_buf.clear();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    /// Creates a new [`Rect`] with the given coordinates and size.
    #[must_use]
    pub const fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
}

impl Color {
    #[must_use]
    pub const fn fg(self) -> [u8; 2] {
        [b'3', b'0' + self as u8]
    }

    #[must_use]
    pub const fn fg_bright(self) -> [u8; 2] {
        [b'9', b'0' + self as u8]
    }

    #[must_use]
    pub fn to_str(x: &[u8]) -> &str {
        core::str::from_utf8(x).unwrap()
    }
}

#[must_use]
pub fn ansi_str_len(s: &str) -> u16 {
    let mut len = 0;
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\x1B' {
            let mut ch = ch;
            while ch != 'm' {
                ch = chars.next().unwrap();
            }
        } else {
            len += 1;
        }
    }
    len
}

#[cfg(test)]
mod tests {
    use super::ansi_str_len;

    #[test]
    fn ansi_len_empty() {
        assert!(ansi_str_len("") == 0);
    }

    #[test]
    fn ansi_len_empty2() {
        assert!(ansi_str_len("\x1B[11121;424m") == 0);
    }

    #[test]
    fn ansi_len_help_text() {
        assert!(ansi_str_len("MOVE WITH \x1B[1;36mARROW KEYS\x1B[0m; EAT \x1B[1;31mFRUIT\x1B[0m; AVOID \x1B[1;32mTAIL\x1B[0m AND \x1B[1;2;37mWALLS\x1B[0m") == 53);
    }
}
