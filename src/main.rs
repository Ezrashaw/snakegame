#![feature(strict_overflow_ops)]

mod snake;
mod terminal;

use std::io;

use snake::game_main;
use terminal::{Color, Key, Rect, Terminal};

const WELCOME_TEXT: &str =
    "Welcome to \x1B[1;32mSNAKE\x1B[0m!\n\n\x1B[2;37mPress \x1B[1m<ENTER>\x1B[22;2m to play!\x1B[0m";

const HELP_TEXT: &str = "MOVE WITH \x1B[1;34mARROW KEYS\x1B[0m; EAT \x1B[1;33mFRUIT\x1B[0m; AVOID \x1B[1;32mTAIL\x1B[0m AND \x1B[1;2;37mWALLS\x1B[0m";
const GAME_OVER_TEXT: &str =
    "GAME OVER!!\x1B[0m\nSCORE: \x1B[1;32m00\x1B[0m\n\n\x1B[2;37mPress \x1B[1m<ENTER>\x1B[22;2m to continue\x1B[0m";

const CANVAS_W: u16 = 61;
const CANVAS_H: u16 = 13;

fn main() -> io::Result<()> {
    let mut terminal = Terminal::new()?;
    let size = terminal.get_termsize();
    let screen_rect = Rect::new(1, 1, size.0 - 2, size.1 - 2);

    let canvas = terminal.draw_canvas(screen_rect, CANVAS_W, CANVAS_H + 2)?;
    let canvas = canvas.change_size(0, -2);
    let textbox = terminal.draw_textbox_centered(canvas, WELCOME_TEXT)?;

    terminal.draw_text_centered(
        Rect::new(canvas.x + 1, canvas.y + CANVAS_H + 2, CANVAS_W, 1),
        HELP_TEXT,
    )?;

    terminal.wait_key(Key::Enter)?;
    terminal.clear_rect(textbox)?;

    let score = game_main(Canvas::new(&mut terminal, canvas))?;
    if let Some(score) = score {
        terminal.write("\x1B[1;91m")?;
        terminal.draw_textbox_centered(
            canvas,
            &GAME_OVER_TEXT.replace("00", &format!("{score:0>2}")),
        )?;
        terminal.wait_key(Key::Enter)?;
    }

    Ok(())
}

struct Canvas<'a> {
    term: &'a mut Terminal,
    rect: Rect,
}

impl<'a> Canvas<'a> {
    fn new(term: &'a mut Terminal, rect: Rect) -> Self {
        Self { term, rect }
    }

    pub fn w(&self) -> u16 {
        self.rect.w / 2
    }

    pub fn h(&self) -> u16 {
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

    pub fn poll_key(&mut self, timeout_ms: u64) -> io::Result<Option<Key>> {
        self.term.poll_key(timeout_ms)
    }

    fn get_xy(&self, coord: Coord) -> (u16, u16) {
        (self.rect.x + 1 + (coord.x * 2), self.rect.y + 1 + coord.y)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Coord {
    x: u16,
    y: u16,
}

impl Coord {
    pub fn as_idx(self, canvas: &Canvas) -> usize {
        self.y as usize * canvas.w() as usize + self.x as usize
    }
}
