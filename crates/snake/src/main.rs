#![feature(array_chunks, let_chains, iter_advance_by)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::module_name_repetitions
)]

mod leaderboard;
mod snake;
mod ui;

use std::{io, time::Duration};

use snake::game_main;
use term::from_pansi;
use ui::GameUi;

const GAME_OVER_TEXT: &str = include_str!("../pansi/game-over.txt");
const WELCOME_TEXT: &str = include_str!("../pansi/welcome.txt");

fn main() -> io::Result<()> {
    let mut ui = GameUi::init()?;

    loop {
        if ui.popup(from_pansi(WELCOME_TEXT), None, true)? {
            break;
        }

        let score = game_main(&mut ui)?;
        if let Some(score) = score {
            ui.term().write("\x1B[1;91m")?;
            if ui.popup(
                from_pansi(GAME_OVER_TEXT).replace("000", &format!("{score:0>3}")),
                Some(Duration::from_secs(10)),
                false,
            )? {
                break;
            }

            ui.reset_game()?;
        } else {
            break;
        }
    }

    Ok(())
}
