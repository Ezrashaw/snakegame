#![feature(array_chunks, let_chains, iter_advance_by)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

mod leaderboard;
mod snake;
mod ui;

use std::{io, time::Duration};

use snake::game_main;
use term::{from_pansi, KeyEvent};
use ui::GameUi;

const GAME_OVER_TEXT: &str = include_str!("../pansi/game-over.txt");
const WELCOME_TEXT: &str = include_str!("../pansi/welcome.txt");

fn main() -> io::Result<()> {
    let mut ui = GameUi::init()?;

    // let mut leaderboard = Leaderboard::init(&mut terminal, canvas)?;
    // if let Some(leaderboard) = &mut leaderboard {
    //     leaderboard.draw_values(&mut terminal)?;
    // }

    loop {
        if ui.popup(from_pansi(WELCOME_TEXT), None, true)? {
            break;
        }

        // if let Some(leaderboard) = &mut leaderboard {
        //     leaderboard.update_you(&mut terminal, 0, true)?;
        // }

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

            // stats.time = Instant::now();
            // stats.update(&mut Canvas::new(&mut terminal, canvas), 0)?;
        } else {
            break;
        }
    }

    Ok(())
}
