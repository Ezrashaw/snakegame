use std::{
    io::{self, Write},
    ptr,
};

use crate::terminal::{
    syscall::{syscall3, SYS_ioctl},
    termios::STDOUT_FD,
};

use super::Terminal;

impl Terminal {
    pub fn write(&mut self, s: &str) -> io::Result<()> {
        write!(self.out, "{}", s)
    }

    pub fn draw_text(&mut self, x: u16, y: u16, s: &str) -> io::Result<()> {
        write!(self.out, "\x1B[{y};{x}H{}", s)
    }

    pub fn draw_pixel(&mut self, x: u16, y: u16, color: Color) -> io::Result<()> {
        write!(self.out, "\x1B[{y};{x}H\x1B[{}m██\x1B[0m", color.as_ansi())
    }

    pub fn clear_pixel(&mut self, x: u16, y: u16) -> io::Result<()> {
        write!(self.out, "\x1B[{y};{x}H  ")
    }

    /// single line only
    pub fn draw_text_centered(&mut self, rect: Rect, s: &str) -> io::Result<()> {
        assert!(rect.w >= ansi_str_len(s));
        assert!(s.lines().count() == 1 && rect.h == 1);

        let x_diff = rect.w - ansi_str_len(s);
        let x_pad = x_diff / 2;
        write!(self.out, "\x1B[{};{}H{}", rect.y, rect.x + x_pad, s)
    }

    pub fn draw_rect_sep(&mut self, rect: Rect, w: u16, h: u16, sep: u16) -> io::Result<Rect> {
        let height_padding = (rect.h - h) / 2;
        let width_padding = (rect.w - w) / 2;
        let x = rect.x + width_padding;
        let y = rect.y + height_padding;
        let w = w as usize;

        writeln!(&mut self.out, "\x1B[{y};{x}H┌{:─<w$}┐", "")?;
        for i in 0..h {
            if i == sep {
                writeln!(&mut self.out, "\x1B[{x}G├{:─<w$}┤", "")?;
            } else {
                writeln!(&mut self.out, "\x1B[{x}G│{:w$}│", "")?;
            }
        }
        write!(&mut self.out, "\x1B[{x}G└{:─<w$}┘", "")?;

        Ok(Rect::new(x, y, w as u16, h))
    }

    /// Draws a box onto the screen.
    ///
    /// The top left border character is at `(x, y)`. The box has an _internal_ height of `h` and an _internal_ width of `w`
    pub fn draw_rect(&mut self, rect: Rect) -> io::Result<()> {
        let (x, y, w, h) = (rect.x, rect.y, rect.w as usize, rect.h);
        writeln!(&mut self.out, "\x1B[{y};{x}H┌{:─<w$}┐", "")?;
        for _ in 0..h {
            writeln!(&mut self.out, "\x1B[{x}G│{:w$}│", "")?;
        }
        write!(&mut self.out, "\x1B[{x}G└{:─<w$}┘", "")
    }

    pub fn draw_rect_centered(&mut self, rect: Rect, w: u16, h: u16) -> io::Result<()> {
        let height_padding = (rect.h - h) / 2;
        let width_padding = (rect.w - w) / 2;
        let x = rect.x + width_padding;
        let y = rect.y + height_padding;

        self.draw_rect(Rect::new(x, y, w, h))
    }

    pub fn draw_textbox(&mut self, x: u16, y: u16, text: &str) -> io::Result<()> {
        let longest = text.lines().map(ansi_str_len).max().unwrap();
        self.draw_rect(Rect::new(x, y, longest + 2, text.lines().count() as u16))?;

        write!(&mut self.out, "\x1B[{};0H", y + 1)?;
        for line in text.lines() {
            let mut col = x + 1;
            // center lines
            let len = ansi_str_len(line);
            if len < longest {
                let diff = longest - len;
                col += diff / 2;
            }

            writeln!(&mut self.out, "\x1B[{col}C{line}")?;
        }

        Ok(())
    }

    /// Draws a textbox that is centered in the imaginary box with top-left
    /// `(x, y)` and size `(w, h)`.
    pub fn draw_textbox_centered(&mut self, rect: Rect, text: &str) -> io::Result<Rect> {
        let tbox_height = text.lines().count() as u16;
        let tbox_width = 2 + text.lines().map(ansi_str_len).max().unwrap();

        let height_padding = (rect.h - tbox_height) / 2;
        let width_padding = (rect.w - tbox_width) / 2;

        let x = rect.x + width_padding;
        let y = rect.y + height_padding;

        self.draw_textbox(x, y, text)?;
        Ok(Rect::new(x, y, tbox_width, tbox_height))
    }

    pub fn clear_rect(&mut self, rect: Rect) -> io::Result<()> {
        let (x, y, w, h) = (rect.x, rect.y, rect.w as usize + 2, rect.h);
        writeln!(&mut self.out, "\x1B[{y};{x}H{:w$}", "")?;
        for _ in 0..h {
            writeln!(&mut self.out, "\x1B[{x}G{:w$}", "")?;
        }
        write!(&mut self.out, "\x1B[{x}G{:w$}", "")
    }

    pub fn get_termsize(&mut self) -> (u16, u16) {
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
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }

    pub fn move_xy(mut self, x: i16, y: i16) -> Self {
        self.x = self.x.strict_add_signed(x);
        self.y = self.y.strict_add_signed(y);
        self
    }

    pub fn change_size(mut self, w: i16, h: i16) -> Self {
        self.w = self.w.strict_add_signed(w);
        self.h = self.h.strict_add_signed(h);
        self
    }
}

pub enum Color {
    Red,
    Green,
    BrightYellow,
    BrightRed,
    Lime,
}

impl Color {
    pub fn as_ansi(&self) -> &'static str {
        match self {
            Color::Red => "31",
            Color::Green => "32",
            Color::BrightYellow => "93",
            Color::BrightRed => "91",
            Color::Lime => "92",
        }
    }
}

fn ansi_str_len(s: &str) -> u16 {
    let mut len = 0;
    let mut i = 0;

    while i < s.len() {
        let mut ch = s.as_bytes()[i];

        if ch == 0x1B {
            while ch != b'm' {
                i += 1;
                ch = s.as_bytes()[i];
            }
        } else {
            len += 1;
        }

        i += 1;
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
