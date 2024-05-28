#![feature(array_chunks, let_chains, iter_advance_by, strict_overflow_ops)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::module_name_repetitions
)]

mod attractor;
mod leaderboard;
mod snake;
mod ui;
mod network;

use std::{io, time::Duration};

use snake::game_main;
use term::{from_pansi, KeyEvent};
use ui::GameUi;

const GAME_OVER_TEXT: &str = include_str!("../pansi/game-over.txt");
const WELCOME_TEXT: &str = include_str!("../pansi/welcome.txt");

fn main() -> io::Result<()> {
    let mut ui = GameUi::init()?;

    loop {
        if ui.popup(from_pansi(WELCOME_TEXT), true, attractor::run)? {
            break;
        }

        ui.reset_game()?;

        match game_main(&mut ui)? {
            Some(score) => {
                ui.term().write("\x1B[1;91m")?;
                if ui.popup(
                    from_pansi(GAME_OVER_TEXT).replace("000", &format!("{score:0>3}")),
                    false,
                    |ctx| {
                        Ok(matches!(
                            ctx.term().wait_enter(Some(Duration::from_secs(10)))?,
                            KeyEvent::Exit
                        ))
                    },
                )? {
                    break;
                }

                ui.reset_game()?;
            }
            None => break,
        }
    }

    Ok(())
}
