#![feature(let_chains, strict_overflow_ops)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::cast_possible_truncation, clippy::module_name_repetitions)]

mod tetris;
mod ui;

use core::{fmt::Write as _, time::Duration};
use oca_game::import_pansi;
use oca_io::{Result, file::File, format, timer::Instant};

use oca_term::{Color, Key, KeyEvent, Popup};
use ui::GameUi;

import_pansi! {
    const GAME_OVER_TEXT = pansi "game-over.txt";
    const WELCOME_TEXT = pansi "welcome.txt";
}

fn main() {
    if let Err(err) = snake_main() {
        writeln!(File::from_fd(2), "\x1B[1;31mBUG\x1B[0m: {err:?}").unwrap();
    }
}

fn snake_main() -> Result<()> {
    let mut ui = GameUi::init()?;

    //loop {
    let popup = Popup::new(WELCOME_TEXT);
    let pos = ui.draw_centered(&popup, false)?;
    ui.flush()?;

    ui.term().wait_enter(None)?;

    tetris::game_main(&mut ui)?;

    ui.clear_centered(&popup, pos)?;
    ui.clear_canvas()?;
    //}

    Ok(())
}
