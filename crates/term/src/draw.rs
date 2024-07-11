use crate::{ansi_str_len, Color, Rect};
use core::fmt::{self, Write};
use oca_io::Result;

#[macro_export]
macro_rules! drawln {
    ($ctx:ident, $($fmt:tt)+) => {{
        use ::core::fmt::Write as _;
        let res = write!($ctx.o(), $($fmt)+);
        res.and_then(|_| drawln!($ctx))
    }};
    ($ctx:ident) => {{
        use ::core::fmt::Write as _;
        let x = $ctx.x();
        write!($ctx.o(), "\n\x1B[{x}G")
    }};
}

#[macro_export]
macro_rules! draw {
    ($ctx:ident, $($fmt:tt)+) => {{
        use ::core::fmt::Write as _;
        write!($ctx.o(), $($fmt)+)
    }};
}

pub trait Draw: Sized {
    type Update = ();
    fn update<W: fmt::Write>(self, _ctx: &mut DrawCtx<W>, _update: Self::Update) -> Result<()> {
        unimplemented!()
    }

    fn size(&self) -> (u16, u16);
    fn draw<W: fmt::Write>(self, ctx: &mut DrawCtx<W>) -> Result<()>;
}

pub struct DrawCtx<'a, W: fmt::Write> {
    out: &'a mut W,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

impl<'a, W: fmt::Write> DrawCtx<'a, W> {
    pub fn o(&mut self) -> &mut (impl fmt::Write + 'a) {
        self.out
    }

    #[must_use]
    pub const fn x(&self) -> u16 {
        self.x
    }

    #[must_use]
    pub const fn size(&self) -> (u16, u16) {
        (self.w, self.h)
    }

    pub fn goto(&mut self, x: u16, y: u16) -> Result<()> {
        assert!(x <= self.w && y <= self.h);
        write!(self.out, "\x1B[{};{}H", self.y + y, self.x + x)?;
        Ok(())
    }

    pub fn draw(&mut self, x: u16, y: u16, object: impl Draw) -> Result<()> {
        draw(&mut self.out, object, self.x + x, self.y + y)
    }
}

fn with_ctx<D: Draw, W: fmt::Write>(
    out: &mut W,
    object: D,
    x: u16,
    y: u16,
    cb: impl FnOnce(&mut DrawCtx<W>, D) -> Result<()>,
) -> Result<()> {
    assert!(x >= 1 && y >= 1);

    write!(out, "\x1B[{y};{x}H")?;
    let (w, h) = object.size();
    let mut ctx = DrawCtx { out, x, y, w, h };

    cb(&mut ctx, object)?;

    Ok(())
}

pub fn draw(out: &mut impl fmt::Write, object: impl Draw, x: u16, y: u16) -> Result<()> {
    with_ctx(out, object, x, y, |ctx, object| object.draw(ctx))
}

pub fn update<T: Draw>(
    out: &mut impl fmt::Write,
    object: T,
    x: u16,
    y: u16,
    update: T::Update,
) -> Result<()> {
    with_ctx(out, object, x, y, |ctx, object| object.update(ctx, update))
}

pub fn draw_centered(
    out: &mut impl fmt::Write,
    object: impl Draw,
    rect: Rect,
    allow_hoff: bool,
) -> Result<(u16, u16)> {
    let (w, h) = object.size();
    assert!(w <= rect.w && h <= rect.h);

    let hoff = (rect.h - h) % 2 != 0;
    if (rect.w - w) % 2 != 0 || (allow_hoff ^ hoff) {
        let w = oca_io::get_termsize().unwrap().0;
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

    fn draw<W: fmt::Write>(self, ctx: &mut DrawCtx<W>) -> Result<()> {
        let str = self.as_ref();
        assert_eq!(str, str.trim_end_matches('\n'));

        for (idx, line) in str.lines().enumerate() {
            if idx != 0 {
                drawln!(ctx)?;
            }
            ctx.o().write_str(line)?;
        }

        Ok(())
    }
}
pub struct CenteredStr<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> Draw for CenteredStr<T> {
    fn size(&self) -> (u16, u16) {
        self.0.size()
    }

    fn draw<W: fmt::Write>(self, ctx: &mut DrawCtx<W>) -> Result<()> {
        let str = self.0.as_ref();
        assert_eq!(str, str.trim_end_matches('\n'));

        let w = self.size().0;

        for (idx, line) in str.lines().enumerate() {
            if idx != 0 {
                drawln!(ctx)?;
            }

            let line_w = ansi_str_len(line);
            let x = (w - line_w) / 2;
            assert!(line_w == 0 || (w - line_w) % 2 == 0, "{idx}");

            if x > 0 {
                draw!(ctx, "\x1B[{x}C")?;
            }
            draw!(ctx, "{line}")?;
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
    const DEFAULT_CORNERS: [char; 4] = ['┌', '┐', '└', '┘'];

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

    fn draw<W: fmt::Write>(self, ctx: &mut DrawCtx<W>) -> Result<()> {
        let (w, h) = (self.w as usize, self.h);
        let corners = self.corners.unwrap_or(Self::DEFAULT_CORNERS);
        let sep = self
            .sep
            .map(|s| (h.strict_add_signed(s) - u16::from(s < 0)) % h);

        drawln!(ctx, "{}{:─<w$}{}", corners[0], "", corners[1])?;
        for i in 0..h {
            if sep.is_some_and(|sep| i == sep) {
                drawln!(ctx, "├{:─<w$}┤", "")?;
            } else if self.clear {
                drawln!(ctx, "│{:w$}│", "")?;
            } else {
                drawln!(ctx, "│\x1B[{w}C│")?;
            }
        }
        draw!(ctx, "{}{:─<w$}{}", corners[2], "", corners[3])?;

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

    fn draw<W: fmt::Write>(self, ctx: &mut DrawCtx<W>) -> Result<()> {
        match self {
            Self::Draw { color, bright } => {
                let color = if bright {
                    color.fg_bright()
                } else {
                    color.fg()
                };
                draw!(ctx, "\x1B[{}m██\x1B[0m", Color::to_str(&color))?;
            }
            Self::Clear => draw!(ctx, "  ")?,
        };
        Ok(())
    }
}

pub struct Popup<'a> {
    text: &'a str,
    color: Option<Color>,
}

impl<'a> Popup<'a> {
    #[must_use]
    pub const fn new(text: &'a str) -> Self {
        Self { text, color: None }
    }

    #[must_use]
    pub fn with_color(mut self, color: Color) -> Self {
        assert!(self.color.is_none());
        self.color = Some(color);
        self
    }
}

impl Draw for &Popup<'_> {
    fn size(&self) -> (u16, u16) {
        let (tw, th) = self.text.size();
        (tw + 4, th + 2)
    }

    fn draw<W: fmt::Write>(self, ctx: &mut DrawCtx<W>) -> Result<()> {
        let (w, h) = ctx.size();
        if let Some(color) = self.color {
            draw!(ctx, "\x1B[1;{}m", Color::to_str(&color.fg_bright()))?;
        }
        ctx.draw(0, 0, Box::new(w - 2, h - 2).with_clear())?;
        ctx.draw(2, 1, CenteredStr(self.text))
    }
}

pub struct Clear(pub u16, pub u16);

impl Draw for Clear {
    fn size(&self) -> (u16, u16) {
        (self.0, self.1)
    }

    fn draw<W: fmt::Write>(self, ctx: &mut DrawCtx<W>) -> Result<()> {
        for _ in 0..self.1 {
            drawln!(ctx, "{:1$}", "", self.0 as usize)?;
        }
        Ok(())
    }
}
