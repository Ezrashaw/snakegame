use std::io::{self, Write};

pub trait Draw {
    type Update = ();
    fn update(&self, ctx: &mut DrawCtx, update: Self::Update) -> io::Result<()> {
        unreachable!()
    }

    fn size(&self) -> (u16, u16);
    fn draw(&self, ctx: &mut DrawCtx) -> io::Result<()>;
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
}

pub fn draw(out: &mut impl Write, object: impl Draw, x: u16, y: u16) -> io::Result<()> {
    let (w, h) = object.size();
    let mut ctx = DrawCtx {
        out: Vec::with_capacity(2048),
        x,
        y,
        w,
        h,
    };

    object.draw(&mut ctx)?;

    let string = String::from_utf8(ctx.out).unwrap();
    let string = string.replace('\n', &format!("\n\x1B[{x}G"));

    write!(out, "\x1B[{y};{x}H{string}")
}

impl Draw for &str {
    type Update = ();

    fn size(&self) -> (u16, u16) {
        let (mut max_width, mut lines) = (0, 0);
        for line in self.lines() {
            lines += 1;
            max_width = max_width.max(crate::stdout::ansi_str_len(line));
        }
        (max_width, lines)
    }

    fn draw(&self, ctx: &mut DrawCtx) -> io::Result<()> {
        let o = ctx.o();
        for (idx, line) in self.lines().enumerate() {
            assert!(!line.trim().is_empty());

            if idx != 0 {
                writeln!(o)?;
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
}

impl Draw for Box {
    type Update = ();

    fn size(&self) -> (u16, u16) {
        (self.w, self.h)
    }

    fn draw(&self, ctx: &mut DrawCtx) -> io::Result<()> {
        let (w, h) = (self.w as usize, self.h);
        let corners = self.corners.unwrap_or(Self::DEFAULT_CORNERS);
        let sep = self
            .sep
            .map(|s| (h.strict_add_signed(s) - u16::from(s < 0)) % h);

        writeln!(ctx.o(), "{}{:─<w$}{}", corners[0], "", corners[1])?;
        for i in 0..h {
            if let Some(sep) = sep
                && i == sep
            {
                writeln!(ctx.o(), "├{:─<w$}┤", "")?;
            } else {
                writeln!(ctx.o(), "│{:w$}│", "")?;
            }
        }
        write!(ctx.o(), "{}{:─<w$}{}", corners[2], "", corners[3])?;

        Ok(())
    }
}
