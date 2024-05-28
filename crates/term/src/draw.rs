use crate::{ansi_str_len, Color, Rect};
use std::io::{self, Write};

pub trait Draw: Sized {
    type Update = ();
    #[allow(unused)]
    fn update(self, ctx: &mut DrawCtx, update: Self::Update) -> io::Result<()> {
        unimplemented!()
    }

    fn size(&self) -> (u16, u16);
    fn draw(self, ctx: &mut DrawCtx) -> io::Result<()>;
}

pub struct DrawCtx {
    out: Vec<u8>,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

impl DrawCtx {
    pub fn o(&mut self) -> &mut impl Write {
        &mut self.out
    }

    pub fn goto(&mut self, x: u16, y: u16) -> io::Result<()> {
        assert!(x <= self.w && y <= self.h);
        write!(self.out, "\x1B[{};{}H", self.y + y, self.x + x)
    }

    pub fn draw(&mut self, x: u16, y: u16, object: impl Draw) -> io::Result<()> {
        draw(&mut self.out, object, self.x + x, self.y + y)
    }
}

fn with_ctx<D: Draw>(
    out: &mut impl Write,
    object: D,
    x: u16,
    y: u16,
    cb: impl FnOnce(&mut DrawCtx, D) -> io::Result<()>,
) -> io::Result<()> {
    assert!(x >= 1 && y >= 1);

    let (w, h) = object.size();
    let mut ctx = DrawCtx {
        out: Vec::with_capacity(2048),
        x,
        y,
        w,
        h,
    };

    cb(&mut ctx, object)?;

    let string: String = String::from_utf8(ctx.out).unwrap();
    let string = string.replace('\n', &format!("\n\x1B[{x}G"));

    write!(out, "\x1B[{y};{x}H{string}")
}

pub fn draw(out: &mut impl Write, object: impl Draw, x: u16, y: u16) -> io::Result<()> {
    with_ctx(out, object, x, y, |ctx, object| object.draw(ctx))
}

pub fn update<T: Draw>(
    out: &mut impl Write,
    object: T,
    x: u16,
    y: u16,
    update: T::Update,
) -> io::Result<()> {
    with_ctx(out, object, x, y, |ctx, object| object.update(ctx, update))
}

pub fn draw_centered(
    out: &mut impl Write,
    object: impl Draw,
    rect: Rect,
    allow_hoff: bool,
) -> io::Result<(u16, u16)> {
    let (w, h) = object.size();
    assert!(w <= rect.w && h <= rect.h);

    if cfg!(feature = "term_debug") {
        write!(out, "\x1B[32m")?;
        draw(out, Box::new(rect.w - 2, rect.h - 2), rect.x, rect.y)?;
        write!(out, "\x1B[0m")?;
    }

    let hoff = (rect.h - h) % 2 != 0;
    if (rect.w - w) % 2 != 0 || (allow_hoff ^ hoff) {
        let w = oca_io::get_termsize().0;
        draw(out, "\x1B[33;1mWARNING: \x1B[0mfailed to center", w - 25, 1)?;
    }

    let x = rect.x + ((rect.w - w) / 2);
    let y = rect.y + ((rect.h - h) / 2);

    draw(out, object, x, y)?;
    Ok((x, y))
}

impl<T: AsRef<str>> Draw for T {
    fn size(&self) -> (u16, u16) {
        let (mut max_width, mut lines) = (0, 0);
        for line in self.as_ref().lines() {
            lines += 1;
            max_width = max_width.max(ansi_str_len(line));
        }
        (max_width, lines)
    }

    fn draw(self, ctx: &mut DrawCtx) -> io::Result<()> {
        let str = self.as_ref();
        assert_eq!(str, str.trim_end_matches('\n'));

        let o = ctx.o();
        for (idx, line) in str.lines().enumerate() {
            if idx != 0 {
                writeln!(o)?;
            }
            write!(o, "{line}")?;
        }

        Ok(())
    }
}
pub struct CenteredStr<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> Draw for CenteredStr<T> {
    fn size(&self) -> (u16, u16) {
        self.0.size()
    }

    fn draw(self, ctx: &mut DrawCtx) -> io::Result<()> {
        let str = self.0.as_ref();
        assert_eq!(str, str.trim_end_matches('\n'));

        let w = self.size().0;
        let o = ctx.o();

        for (idx, line) in str.lines().enumerate() {
            if idx != 0 {
                writeln!(o)?;
            }

            let line_w = ansi_str_len(line);
            let x = (w - line_w) / 2;
            assert!((w - line_w) % 2 == 0);

            if x > 0 {
                write!(o, "\x1B[{x}C")?;
            }
            write!(o, "{line}")?;
        }

        Ok(())
    }
}

pub struct Box {
    w: u16,
    h: u16,
    sep: Option<i16>,
    corners: Option<[char; 4]>,
    clear: bool,
}

impl Box {
    pub const DEFAULT_CORNERS: [char; 4] = ['┌', '┐', '└', '┘'];

    #[must_use]
    pub const fn new(w: u16, h: u16) -> Self {
        Self {
            w,
            h,
            sep: None,
            corners: None,
            clear: false,
        }
    }

    /// Adds a seperator line into the [`Box`].
    ///
    /// Value represents the number of lines before the seperator. A negative number positions the
    /// separator from the bottom, rather than the top.
    ///
    /// # Panics
    ///
    /// - If `sep == 0`, then this this function panics
    /// - If the absolute value of `sep` is more than the height of the box, then this function
    ///   panics.
    #[must_use]
    pub const fn with_separator(mut self, sep: i16) -> Self {
        assert!(self.sep.is_none());
        assert!(sep != 0);
        assert!(sep.unsigned_abs() < self.h);

        self.sep = Some(sep);
        self
    }

    #[must_use]
    pub const fn with_corners(mut self, corners: [char; 4]) -> Self {
        assert!(self.corners.is_none());
        self.corners = Some(corners);
        self
    }

    #[must_use]
    pub const fn with_clear(mut self) -> Self {
        assert!(!self.clear);
        self.clear = true;
        self
    }
}

impl Draw for Box {
    fn size(&self) -> (u16, u16) {
        (self.w + 2, self.h + 2)
    }

    fn draw(self, ctx: &mut DrawCtx) -> io::Result<()> {
        let (w, h) = (self.w as usize, self.h);
        let corners = self.corners.unwrap_or(Self::DEFAULT_CORNERS);
        let sep = self
            .sep
            .map(|s| (h.strict_add_signed(s) - u16::from(s < 0)) % h);

        writeln!(ctx.o(), "{}{:─<w$}{}", corners[0], "", corners[1])?;
        for i in 0..h {
            if sep.is_some_and(|sep| i == sep) {
                writeln!(ctx.o(), "├{:─<w$}┤", "")?;
            } else if self.clear {
                writeln!(ctx.o(), "│{:w$}│", "")?;
            } else {
                writeln!(ctx.o(), "│\x1B[{w}C│")?;
            }
        }
        write!(ctx.o(), "{}{:─<w$}{}", corners[2], "", corners[3])?;

        Ok(())
    }
}

pub enum Pixel {
    Draw { color: Color, bright: bool },
    Clear,
}

impl Pixel {
    #[must_use]
    pub const fn new(color: Color, bright: bool) -> Self {
        Self::Draw { color, bright }
    }
}

impl Draw for Pixel {
    fn size(&self) -> (u16, u16) {
        (2, 1)
    }

    fn draw(self, ctx: &mut DrawCtx) -> io::Result<()> {
        match self {
            Self::Draw { color, bright } => {
                let color = if bright {
                    color.fg_bright()
                } else {
                    color.fg()
                };
                let color = Color::to_str(&color);
                ctx.draw(0, 0, format!("\x1B[{color}m██\x1B[0m"))
            }
            Self::Clear => ctx.draw(0, 0, "  "),
        }
    }
}
