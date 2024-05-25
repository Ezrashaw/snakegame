use crate::Draw;

use super::Terminal;
use std::io::{self, Write};

impl Terminal {
    pub const DEFAULT_CORNERS: [char; 4] = ['┌', '┐', '└', '┘'];

    pub fn write(&mut self, s: &str) -> io::Result<()> {
        write!(self.out, "{s}")
    }

    pub fn draw(&mut self, x: u16, y: u16, object: impl Draw) -> io::Result<()> {
        crate::draw(&mut self.out, object, x, y)
    }

    pub fn update<T: Draw>(&mut self, x: u16, y: u16, object: T, u: T::Update) -> io::Result<()> {
        crate::update(&mut self.out, object, x, y, u)
    }

    pub fn draw_centered(&mut self, object: impl Draw, rect: Rect) -> io::Result<(u16, u16)> {
        crate::draw_centered(&mut self.out, object, rect, false)
    }

    pub fn draw_centered_hoff(
        &mut self,
        object: impl Draw,
        rect: Rect,
        hoff: bool,
    ) -> io::Result<(u16, u16)> {
        crate::draw_centered(&mut self.out, object, rect, hoff)
    }

    pub fn draw_pixel(&mut self, x: u16, y: u16, color: Color) -> io::Result<()> {
        write!(self.out, "\x1B[{y};{x}H\x1B[{}m██\x1B[0m", color.as_ansi())
    }

    pub fn clear_pixel(&mut self, x: u16, y: u16) -> io::Result<()> {
        write!(self.out, "\x1B[{y};{x}H  ")
    }

    pub fn clear_rect(&mut self, rect: Rect) -> io::Result<()> {
        let (x, y, w, h) = (rect.x, rect.y, rect.w as usize, rect.h);
        for i in 0..h {
            write!(&mut self.out, "\x1B[{};{x}H{:w$}", y + i, "")?;
        }
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

    #[must_use]
    pub fn move_xy(mut self, x: i16, y: i16) -> Self {
        self.x = self.x.strict_add_signed(x);
        self.y = self.y.strict_add_signed(y);
        self
    }

    #[must_use]
    pub fn change_size(mut self, w: i16, h: i16) -> Self {
        self.w = self.w.strict_add_signed(w);
        self.h = self.h.strict_add_signed(h);
        self
    }
}

#[derive(Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    White,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightMagenta,
    BrightCyan,
}

impl Color {
    #[must_use]
    pub const fn as_ansi(self) -> &'static str {
        match self {
            Self::Red => "31",
            Self::Green => "32",
            Self::Yellow => "33",
            Self::Blue => "34",
            Self::Magenta => "35",
            Self::White => "37",
            Self::BrightRed => "91",
            Self::BrightGreen => "92",
            Self::BrightYellow => "93",
            Self::BrightMagenta => "95",
            Self::BrightCyan => "96",
        }
    }
}
