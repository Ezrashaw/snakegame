use std::{
    io::{self, Write},
    time::Instant,
};

use term::{ansi_str_len, from_pansi, Box, CenteredStr, Color, Draw, DrawCtx, Rect, Terminal};

use crate::leaderboard::Leaderboard;

const CREDITS_TEXT: &str = include_str!("../pansi/credits.txt");
const STATS_TEXT: &str = include_str!("../pansi/stats.txt");
const SNAKE_TEXT: &str = include_str!("../pansi/snake.txt");
const HELP_TEXT: &str = include_str!("../pansi/help.txt");
const GIT_TEXT: &str = include_str!(concat!(env!("OUT_DIR"), "/git.txt"));

pub const CANVAS_W: u16 = 28;
pub const CANVAS_H: u16 = 18;

pub struct GameUi {
    term: Terminal,
    stats: Stats,
    lb: Option<Leaderboard>,
    cx: u16,
    cy: u16,
}

impl GameUi {
    pub fn init() -> io::Result<Self> {
        let mut term = Terminal::new()?;

        let (cx, cy) = Self::draw_static(&mut term)?;

        let stats = Stats(Instant::now());
        term.draw(cx - 16, cy + 2, &stats)?;

        let mut lb = Leaderboard::init();
        if let Some(lb) = &mut lb {
            term.draw(cx + (CANVAS_W * 2) + 4, cy, lb)?;
        }

        Ok(Self {
            term,
            stats,
            lb,
            cx,
            cy,
        })
    }

    pub fn popup<T>(
        &mut self,
        text: impl AsRef<str>,
        hoff: bool,
        f: impl FnOnce(&mut Self) -> io::Result<T>,
    ) -> io::Result<T> {
        let (w, h) = text.size();

        let popup = Box::new(w + 2, h).with_clear();
        let (px, py) = self.term.draw_centered_hoff(
            popup,
            Rect::new(self.cx, self.cy, (CANVAS_W * 2) + 2, CANVAS_H + 2),
            hoff,
        )?;
        self.term.draw(px + 2, py + 1, CenteredStr(text))?;

        let ret = f(self)?;

        self.term.clear_rect(Rect::new(px, py, w + 4, h + 2))?;

        Ok(ret)
    }

    pub fn draw_pixel(&mut self, coord: Coord, color: Color) -> io::Result<()> {
        self.term
            .draw_pixel(self.cx + (coord.x * 2) + 1, self.cy + coord.y + 1, color)
    }

    pub fn clear_pixel(&mut self, coord: Coord) -> io::Result<()> {
        self.term
            .clear_pixel(self.cx + (coord.x * 2) + 1, self.cy + coord.y + 1)
    }

    pub fn update(&mut self, score: usize) -> io::Result<()> {
        self.term
            .update(self.cx - 16, self.cy + 2, &self.stats, score)?;

        if let Some(lb) = &mut self.lb {
            self.term.update(
                self.cx + (CANVAS_W * 2) + 4,
                self.cy,
                lb,
                (score.try_into().unwrap(), false),
            )?;
        }

        Ok(())
    }

    pub fn reset_game(&mut self) -> io::Result<()> {
        self.term
            .clear_rect(Rect::new(self.cx + 1, self.cy + 1, CANVAS_W * 2, CANVAS_H))?;

        self.stats.0 = Instant::now();
        self.term
            .update(self.cx - 16, self.cy + 2, &self.stats, 0)?;

        if let Some(lb) = &mut self.lb {
            self.term
                .update(self.cx + (CANVAS_W * 2) + 4, self.cy, lb, (0, true))?;
        }

        Ok(())
    }

    pub fn term(&mut self) -> &mut Terminal {
        &mut self.term
    }

    /// This function draws all the "static" elements to the screen. These elements do not change or
    /// update in any way throughout the runtime of the program.
    ///
    /// Currently this includes:
    /// - The credits (bottom left corner);
    /// - The git commit text (bottom right corner);
    /// - The SNAKE text (top center);
    /// - The canvas/play area; and
    /// - The help text (beneath canvas).
    fn draw_static(term: &mut Terminal) -> io::Result<(u16, u16)> {
        let (w, h) = term.size();

        // Draw the credits text in the bottom left corner of the screen.
        term.draw(1, h - 3, from_pansi(CREDITS_TEXT))?;

        // Draw the git commit text in the bottom right corner of the screen.
        let git_text = from_pansi(GIT_TEXT);
        let git_width = ansi_str_len(git_text.split_once('\n').unwrap().0);
        term.draw(w - git_width as u16, h - 1, git_text)?;

        // Draw the SNAKE title text in the top center of the screen.
        term.draw_centered(from_pansi(SNAKE_TEXT), Rect::new(1, 1, w, 4))?;

        // Draw the outline of the canvas in the center of the entire screen. We use the xy values
        // given back to calculate the position of the help text, and the leaderboard + stats panel
        // but the latter are in other places.
        let (cx, cy) = term.draw_centered(
            Box::new(CANVAS_W * 2, CANVAS_H + 3).with_separator(-2),
            Rect::new(1, 1, w, h),
        )?;

        // Draw the help text, centered underneath the canvas.
        term.draw_centered(
            CenteredStr(from_pansi(HELP_TEXT)),
            Rect::new(cx + 1, cy + CANVAS_H + 2, CANVAS_W * 2, 2),
        )?;

        // Return the canvas coordinates so that other UI elements can use them.
        Ok((cx, cy))
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
}

impl Coord {
    pub const fn as_idx(self) -> usize {
        self.y as usize * CANVAS_W as usize + self.x as usize
    }
}

struct Stats(Instant);

impl Draw for &Stats {
    fn size(&self) -> (u16, u16) {
        (15, 4)
    }

    fn draw(self, ctx: &mut DrawCtx) -> io::Result<()> {
        ctx.draw(
            0,
            0,
            Box::new(15, 4)
                .with_separator(1)
                .with_corners(['┌', '┤', '└', '┤']),
        )?;
        ctx.draw(2, 1, from_pansi(STATS_TEXT))
    }

    type Update = usize;
    fn update(self, ctx: &mut DrawCtx, update: Self::Update) -> io::Result<()> {
        let t = self.0.elapsed();
        let mins = t.as_secs() / 60;
        let secs = t.as_secs() % 60;

        ctx.goto(12, 3)?;
        write!(ctx.o(), "{update:0>3}")?;
        ctx.goto(10, 4)?;
        write!(ctx.o(), "{mins:0>2}:{secs:0>2}")?;

        Ok(())
    }
}
