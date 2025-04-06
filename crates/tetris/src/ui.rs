use core::fmt;

use oca_game::import_pansi;
use oca_term::{Box, CenteredStr, Clear, Draw, DrawCtx, Rect, Terminal, draw};

use oca_io::{Result, timer::Instant};

import_pansi! {
    const CREDITS_TEXT = pansi "credits.txt";
    const STATS_TEXT = pansi "stats.txt";
    const TETRIS_TEXT = pansi "tetris.txt";
    const HELP_TEXT = pansi "help.txt";
    #[cfg(debug_assertions)]
    const GIT_TEXT = pansi "git.txt";
}

pub const CANVAS_W: u16 = 20;
pub const CANVAS_H: u16 = 20;

pub struct GameUi {
    term: Terminal,
    stats: Stats,
    cx: u16,
    cy: u16,
}

impl GameUi {
    pub fn init() -> Result<Self> {
        let mut term = Terminal::new()?;
        let size = term.size();
        // FIXME: update for tetris
        if size.0 < 95 || size.1 < 33 {
            term.exit_with_error("terminal is too small; (95, 33) required")
        }
        let (cx, cy) = draw_static(&mut term)?;

        let stats = Stats(Instant::now()?);
        term.draw(cx - 16, cy + 2, &stats)?;

        Ok(Self {
            term,
            stats,
            cx,
            cy,
        })
    }

    pub fn draw_centered(&mut self, object: impl Draw, hoff: bool) -> Result<(u16, u16)> {
        self.term.draw_centered_hoff(
            object,
            Rect::new(self.cx, self.cy, (CANVAS_W * 2) + 2, CANVAS_H + 2),
            hoff,
        )
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn clear_centered(&mut self, object: impl Draw, pos: (u16, u16)) -> Result<()> {
        let (w, h) = object.size();
        self.term.draw(pos.0, pos.1, Clear(w, h))
    }

    pub fn draw_canvas(&mut self, coord: Coord, object: impl Draw) -> Result<()> {
        self.term
            .draw(self.cx + (coord.x * 2) + 1, self.cy + coord.y + 1, object)
    }

    pub fn update_score(&mut self, score: usize) -> Result<()> {
        self.update_stats(StatsUpdate::Score(score))
    }

    pub fn update_tick(&mut self, stats: bool) -> Result<bool> {
        if stats {
            self.term
                .update(self.cx - 16, self.cy + 2, &self.stats, StatsUpdate::Time)?;
        }

        self.term.process_signals()
    }

    pub fn clear_canvas(&mut self) -> Result<()> {
        self.term
            .draw(self.cx + 1, self.cy + 1, Clear(CANVAS_W * 2, CANVAS_H))
    }

    pub fn reset_stats(&mut self) -> Result<()> {
        self.stats.0 = Instant::now()?;
        self.update_stats(StatsUpdate::Time)?;
        self.update_stats(StatsUpdate::Score(0))
    }

    pub fn term(&mut self) -> &mut Terminal {
        &mut self.term
    }

    fn update_stats(&mut self, up: StatsUpdate) -> Result<()> {
        self.term.update(self.cx - 16, self.cy + 2, &self.stats, up)
    }

    pub fn flush(&mut self) -> Result<()> {
        self.term.flush()
    }
}

/// This function draws all the "static" elements to the screen. These elements do not change or
/// update in any way throughout the runtime of the program.
///
/// Currently this includes:
/// - The credits (bottom left corner);
/// - The git commit text (bottom right corner);
/// - The TETRIS text (top center);
/// - The canvas/play area; and
/// - The help text (beneath canvas).
fn draw_static(term: &mut Terminal) -> Result<(u16, u16)> {
    let (w, h) = term.size();

    // Draw the credits text in the bottom left corner of the screen.
    term.draw(1, h - 3, CREDITS_TEXT)?;

    // Draw the git commit text in the bottom right corner of the screen, but only when compiling
    // for debug mode.
    #[cfg(debug_assertions)]
    {
        let git_width = oca_term::ansi_str_len(GIT_TEXT.split_once('\n').unwrap().0);
        term.draw(w - git_width, h - 1, GIT_TEXT)?;
    }

    // Draw the TETRIS title text in the top center of the screen.
    term.draw_centered(TETRIS_TEXT, Rect::new(1, 1, w, 4))?;

    // Draw the outline of the canvas in the center of the entire screen. We use the xy values
    // given back to calculate the position of the help text, and the leaderboard + stats panel
    // but the latter are in other places.
    let (cx, cy) = term.draw_centered_hoff(
        Box::new(CANVAS_W * 2, CANVAS_H + 3).with_horz_lines(&[-2]),
        Rect::new(1, 1, w, h),
        true,
    )?;

    // Draw the help text, centered underneath the canvas.
    term.draw_centered(
        CenteredStr(HELP_TEXT),
        Rect::new(cx + 1, cy + CANVAS_H + 2, CANVAS_W * 2, 2),
    )?;

    // Return the canvas coordinates so that other UI elements can use them.
    Ok((cx, cy))
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

    pub const fn moved(self, x: i16, y: i16) -> Self {
        Self {
            x: self.x.strict_add_signed(x),
            y: self.y.strict_add_signed(y),
        }
    }
}

struct Stats(Instant);

impl Draw for &Stats {
    fn size(&self) -> (u16, u16) {
        (15, 4)
    }

    fn draw<W: fmt::Write>(self, ctx: &mut DrawCtx<W>) -> Result<()> {
        ctx.draw(
            0,
            0,
            Box::new(15, 4)
                .with_horz_lines(&[1])
                .with_corners(['┌', '┤', '└', '┤']),
        )?;
        ctx.draw(2, 1, STATS_TEXT)
    }

    type Update = StatsUpdate;
    fn update<W: fmt::Write>(self, ctx: &mut DrawCtx<W>, update: Self::Update) -> Result<()> {
        match update {
            StatsUpdate::Score(score) => {
                ctx.goto(12, 3)?;
                draw!(ctx, "{score:0>3}")?;
            }
            StatsUpdate::Time => {
                let t = Instant::now()? - self.0;
                let mins = t.as_secs() / 60;
                let secs = t.as_secs() % 60;

                ctx.goto(10, 4)?;
                draw!(ctx, "{mins:0>2}:{secs:0>2}")?;
            }
        };
        Ok(())
    }
}

enum StatsUpdate {
    Score(usize),
    Time,
}
