#![feature(
    array_chunks,
    let_chains,
    iter_advance_by,
    strict_overflow_ops,
    if_let_guard
)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::module_name_repetitions
)]

mod attractor;
mod leaderboard;
mod network;
mod snake;
mod ui;

use std::{io, time::Duration};

use snake::game_main;
use term::{from_pansi, Color, KeyEvent, Popup};
use ui::GameUi;

const GAME_OVER_TEXT: &str = include_str!("../pansi/game-over.txt");
const WELCOME_TEXT: &str = include_str!("../pansi/welcome.txt");

fn main() -> io::Result<()> {
    let mut ui = GameUi::init()?;

    loop {
        let welcome_text = from_pansi(WELCOME_TEXT);
        let popup = Popup::new(&welcome_text);
        let pos = ui.draw_centered(&popup, true)?;
        if attractor::run(&mut ui)? {
            break;
        }
        ui.clear_centered(&popup, pos)?;

        ui.reset_game()?;

        match game_main(&mut ui)? {
            Some(score) => {
                let game_over_text =
                    from_pansi(GAME_OVER_TEXT).replace("000", &format!("{score:0>3}"));
                let popup = Popup::new(&game_over_text).with_color(Color::Red);
                let pos = ui.draw_centered(&popup, false)?;
                if ui.term().wait_enter(Some(Duration::from_secs(10)))? == KeyEvent::Exit {
                    break;
                }
                ui.clear_centered(&popup, pos)?;

                ui.reset_game()?;
            }
            None => break,
        }
    }

    Ok(())
}
