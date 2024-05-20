#![feature(array_chunks, let_chains, iter_advance_by)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

mod leaderboard;
mod snake;

use std::{io, time::Instant};

use leaderboard::Leaderboard;
use snake::game_main;
use term::{from_pansi, Color, Key, KeyEvent, Rect, Terminal};

const WELCOME_TEXT: &str = include_str!("../welcome.txt");
const HELP_TEXT: &str = include_str!("../help.txt");
const GAME_OVER_TEXT: &str = include_str!("../game-over.txt");
const CREDITS_TEXT: &str = include_str!("../credits.txt");
const STATS_TEXT: &str = include_str!("../stats.txt");
const SNAKE_TEXT: &str = include_str!("../snake.txt");
const GIT_TEXT: &str = include_str!(concat!(env!("OUT_DIR"), "/git.txt"));

const CANVAS_W: u16 = 56;
const CANVAS_H: u16 = 17;

fn main() -> io::Result<()> {
    let mut terminal = Terminal::new()?;
    let size = term::get_termsize();
    let screen_rect = Rect::new(1, 1, size.0 - 2, size.1 - 2);

    terminal.draw_text(0, size.1 - 3, &from_pansi(CREDITS_TEXT))?;

    let half_git = GIT_TEXT.len() / 2;
    let half_git = GIT_TEXT[half_git..].find(' ').unwrap() + half_git;
    let (fh, sh) = GIT_TEXT.split_at(half_git);
    terminal.write("\x1B[2m")?;
    terminal.draw_text(size.0 - fh.len() as u16, size.1 - 1, fh)?;
    terminal.draw_text(size.0 - fh.len() as u16, size.1, &sh[1..])?;
    terminal.write("\x1B[0m")?;

    let canvas = terminal.draw_rect_sep(
        screen_rect,
        CANVAS_W,
        CANVAS_H + 3,
        CANVAS_H,
        Terminal::DEFAULT_CORNERS,
    )?;
    let canvas = canvas.change_size(0, -3);

    let stats_rect = terminal.draw_rect_sep(
        Rect::new(canvas.x - 16, canvas.y + 2, 16, 5),
        15,
        4,
        1,
        ['┌', '┤', '└', '┤'],
    )?;
    terminal.draw_text_centered(stats_rect.move_xy(1, 1), "\x1B[1;33mSTATS\x1B[0m")?;
    terminal.draw_text_centered(
        stats_rect.move_xy(1, 3).change_size(0, 1),
        &from_pansi(STATS_TEXT),
    )?;

    terminal.draw_text_centered(
        Rect::new(canvas.x, 1, CANVAS_W + 2, 5),
        &from_pansi(SNAKE_TEXT),
    )?;

    terminal.draw_text_centered(
        Rect::new(canvas.x + 1, canvas.y + CANVAS_H + 2, CANVAS_W, 2),
        &from_pansi(HELP_TEXT),
    )?;

    let mut leaderboard = Leaderboard::init(&mut terminal, canvas)?;
    if let Some(leaderboard) = &mut leaderboard {
        leaderboard.draw_values(&mut terminal)?;
    }

    loop {
        let textbox = terminal.draw_textbox_centered(canvas, &from_pansi(WELCOME_TEXT))?;
        if terminal.wait_key(|k| k == Key::Enter, None, true)? == KeyEvent::Exit {
            break;
        }
        terminal.clear_rect(textbox)?;

        if let Some(leaderboard) = &mut leaderboard {
            leaderboard.update_you(&mut terminal, 0, true)?;
        }

        let stats = Stats {
            time: Instant::now(),
        };

        let score = game_main(Canvas::new(&mut terminal, canvas), &mut leaderboard, &stats)?;
        if let Some(score) = score {
            terminal.write("\x1B[1;91m")?;
            terminal.draw_textbox_centered(
                canvas,
                &from_pansi(GAME_OVER_TEXT).replace("000", &format!("{score:0>3}")),
            )?;
            if terminal.wait_key(|k| k == Key::Enter, Some(10_000), true)? == KeyEvent::Exit {
                break;
            }
            terminal.clear_rect(canvas.move_xy(1, 1).change_size(-2, -2))?;

            terminal.draw_text_centered(
                stats_rect.move_xy(1, 3).change_size(0, 1),
                &from_pansi(STATS_TEXT),
            )?;
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
        self.term.wait_key(want_key, timeout_ms, false)
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

struct Stats {
    time: Instant,
}

impl Stats {
    pub fn update(&self, canvas: &mut Canvas, score: usize) -> io::Result<()> {
        let t = self.time.elapsed();
        let mins = t.as_secs() / 60;
        let secs = t.as_secs() % 60;

        canvas.term.write(&format!(
            "\x1B[{};{}H{:0>3}\n\x1B[{}G{:0>2}:{:0>2}",
            canvas.rect.y + 5,
            canvas.rect.x - 4,
            score,
            canvas.rect.x - 6,
            mins,
            secs,
        ))?;

        Ok(())
    }
}
