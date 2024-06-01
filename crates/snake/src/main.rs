#![feature(let_chains)]
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

use std::{
    io,
    time::{Duration, Instant},
};

use snake::game_main;
use term::{Color, Key, KeyEvent, Popup};
use ui::GameUi;

const GAME_OVER_TEXT: &str = include_str!(concat!(env!("OUT_DIR"), "/game-over.txt"));
const ADD_LB_TEXT: &str = include_str!(concat!(env!("OUT_DIR"), "/add-lb.txt"));
const WELCOME_TEXT: &str = include_str!(concat!(env!("OUT_DIR"), "/welcome.txt"));

fn main() {
    snake_main().unwrap();
}

fn snake_main() -> io::Result<()> {
    let mut ui = GameUi::init()?;

    loop {
        let popup = Popup::new(WELCOME_TEXT);
        let pos = ui.draw_centered(&popup, false)?;
        if attractor::run(&mut ui)? {
            break;
        }
        ui.clear_centered(&popup, pos)?;

        ui.reset_game(Some(0))?;

        match game_main(&mut ui)? {
            Some(score) => {
                if ui.network().is_some() && score > 3 {
                    do_highscore(&mut ui, score)?;
                } else {
                    let game_over_text = GAME_OVER_TEXT.replace("000", &format!("{score:0>3}"));
                    let popup = Popup::new(&game_over_text).with_color(Color::Red);
                    let pos = ui.draw_centered(&popup, true)?;
                    if ui.term().wait_enter(Some(Duration::from_secs(10)))? == KeyEvent::Exit {
                        break;
                    }
                    ui.clear_centered(&popup, pos)?;
                }

                ui.reset_game(None)?;
            }
            None => break,
        }
    }

    Ok(())
}

fn do_highscore(ui: &mut GameUi, score: usize) -> io::Result<()> {
    ui.term().clear_input()?;

    let game_over_text = ADD_LB_TEXT.replace("000", &format!("{score:0>3}"));

    let popup = Popup::new(&game_over_text).with_color(Color::Green);
    let pos = ui.draw_centered(&popup, false)?;
    let mut colored_left = true;
    let mut next_update = Instant::now() + Duration::from_millis(500);
    let mut cursor_pos = 0;
    let mut input = [0u8; 3];

    loop {
        match ui
            .term()
            .get_key_timeout(Some(next_update.duration_since(Instant::now())), |k| {
                matches!(k, Key::Char(_) | Key::Back | Key::Enter)
            })? {
            Some(Key::Char(ch)) if cursor_pos < 3 => {
                let ch = ch.to_ascii_uppercase();
                input[cursor_pos as usize] = ch;
                cursor_pos += 1;
                ui.term().draw(
                    pos.0 + 9 + cursor_pos,
                    pos.1 + 5,
                    format!("\x1B[1m{}\x1B[0m", ch as char),
                )?;
            }
            Some(Key::Back) if cursor_pos > 0 => {
                ui.term()
                    .draw(pos.0 + 9 + cursor_pos, pos.1 + 5, "\x1B[2m-\x1B[0m")?;
                cursor_pos -= 1;
            }
            Some(Key::Enter) if cursor_pos == 3 => break,
            _ => (),
        }

        if Instant::now() > next_update {
            let str = if colored_left {
                "\x1B[32mGREAT \x1B[1mSCORE\x1B[0m"
            } else {
                "\x1B[32;1mGREAT \x1B[22mSCORE\x1B[0m"
            };
            ui.term().draw(pos.0 + 11, pos.1 + 1, str)?;
            colored_left = !colored_left;
            next_update = Instant::now() + Duration::from_millis(500);
        }
    }

    ui.clear_centered(&popup, pos)?;

    ui.network().unwrap().send_game(input, score as u8)?;
    ui.block_update_lb()?;

    Ok(())
}
