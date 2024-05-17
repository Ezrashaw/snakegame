#![feature(strict_overflow_ops, array_chunks, if_let_guard, let_chains)]

#[cfg(not(all(target_os = "linux",)))]
compile_error!("This program only runs on Linux");

mod leaderboard;
mod snake;
mod terminal;

use std::io;

use leaderboard::Leaderboard;
use snake::game_main;
use terminal::{Color, Key, KeyEvent, Rect, Terminal};

const WELCOME_TEXT: &str =
    "Welcome to \x1B[1;32mSNAKE\x1B[0m!\n\n\x1B[2;37mPress \x1B[1m<ENTER>\x1B[22;2m to play!\x1B[0m";

const HELP_TEXT: &str = "MOVE WITH \x1B[1;34mARROW KEYS\x1B[0m OR \x1B[1;34mWASD\x1B[0m\nEAT \x1B[1;93mFRUIT\x1B[0m ; AVOID \x1B[1;32mTAIL\x1B[0m AND \x1B[1;2;37mWALLS\x1B[0m";

const GAME_OVER_TEXT: &str =
    "GAME OVER!\x1B[0m\nSCORE: \x1B[1;93m000\x1B[0m\n\n\x1B[2;37mPress \x1B[1m<ENTER>\x1B[22;2m to continue...\x1B[0m";

const CREDITS_TEXT: &str = include_str!("../credits.txt");

const CANVAS_W: u16 = 56;
const CANVAS_H: u16 = 17;

fn main() -> io::Result<()> {
    let mut terminal = Terminal::new()?;
    let size = terminal::get_termsize();
    let screen_rect = Rect::new(1, 1, size.0 - 2, size.1 - 2);

    terminal.draw_text(0, size.1 - 3, CREDITS_TEXT)?;

    let canvas = terminal.draw_rect_sep(screen_rect, CANVAS_W, CANVAS_H + 3, CANVAS_H)?;
    let canvas = canvas.change_size(0, -3);

    terminal.draw_text_centered(
        Rect::new(canvas.x + 1, canvas.y + CANVAS_H + 2, CANVAS_W, 2),
        HELP_TEXT,
    )?;

    let mut leaderboard = Leaderboard::init(&mut terminal, canvas)?;
    if let Some(leaderboard) = &mut leaderboard {
        leaderboard.draw_values(&mut terminal)?;
    }

    loop {
        let textbox = terminal.draw_textbox_centered(canvas, WELCOME_TEXT)?;
        terminal.wait_key(|k| k == Key::Enter, None)?;
        terminal.clear_rect(textbox)?;

        if let Some(leaderboard) = &mut leaderboard {
            leaderboard.update_you(&mut terminal, 0)?;
        }

        let score = game_main(Canvas::new(&mut terminal, canvas), &mut leaderboard)?;
        if let Some(score) = score {
            terminal.write("\x1B[1;91m")?;
            terminal.draw_textbox_centered(
                canvas,
                &GAME_OVER_TEXT.replace("000", &format!("{score:0>3}")),
            )?;
            terminal.wait_key(|k| k == Key::Enter, Some(10_000))?;
            terminal.clear_rect(canvas.move_xy(1, 1).change_size(-2, -2))?;
        } else {
            break;
        }
    }

    Ok(())
}

struct Canvas<'a> {
    term: &'a mut Terminal,
    rect: Rect,
}

impl<'a> Canvas<'a> {
    pub fn new(term: &'a mut Terminal, rect: Rect) -> Self {
        Self { term, rect }
    }

    pub const fn w(&self) -> u16 {
        self.rect.w / 2
    }

    pub const fn h(&self) -> u16 {
        self.rect.h
    }

    pub fn draw_pixel(&mut self, coord: Coord, color: Color) -> io::Result<()> {
        let (x, y) = self.get_xy(coord);
        self.term.draw_pixel(x, y, color)
    }

    pub fn clear_pixel(&mut self, coord: Coord) -> io::Result<()> {
        let (x, y) = self.get_xy(coord);
        self.term.clear_pixel(x, y)
    }

    pub fn wait_key(
        &mut self,
        want_key: impl Fn(Key) -> bool,
        timeout_ms: Option<u64>,
    ) -> io::Result<KeyEvent> {
        self.term.wait_key(want_key, timeout_ms)
    }

    const fn get_xy(&self, coord: Coord) -> (u16, u16) {
        (self.rect.x + 1 + (coord.x * 2), self.rect.y + 1 + coord.y)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Coord {
    x: u16,
    y: u16,
}

impl Coord {
    pub const fn as_idx(self, canvas: &Canvas) -> usize {
        self.y as usize * canvas.w() as usize + self.x as usize
    }
}
