#![feature(let_chains)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::cast_possible_truncation, clippy::module_name_repetitions)]

mod attractor;
mod leaderboard;
mod snake;
mod ui;

use core::{fmt::Write as _, time::Duration};
use oca_io::{file::File, format, timer::Instant, Result};

use snake::game_main;
use term::{Color, Key, KeyEvent, Popup};
use ui::GameUi;

const GAME_OVER_TEXT: &str = include_str!(concat!(env!("OUT_DIR"), "/game-over.txt"));
const ADD_LB_TEXT: &str = include_str!(concat!(env!("OUT_DIR"), "/add-lb.txt"));
const WELCOME_TEXT: &str = include_str!(concat!(env!("OUT_DIR"), "/welcome.txt"));

fn main() {
    if let Err(err) = snake_main() {
        writeln!(File::from_fd(2), "\x1B[1;31mBUG\x1B[0m: {err:?}").unwrap();
    }
}

fn snake_main() -> Result<()> {
    let mut ui = GameUi::init()?;

    loop {
        let popup = Popup::new(WELCOME_TEXT);
        let pos = ui.draw_centered(&popup, false)?;
        if attractor::run(&mut ui)? {
            break;
        }
        ui.clear_centered(&popup, pos)?;
        ui.clear_canvas()?;
        if let Some(lb) = ui.lb() {
            lb.score = Some(0);
            ui.reset_lb(false)?;
        }

        match game_main(&mut ui)? {
            Some(score) => {
                let needs_lb_update = if let Some(lb) = ui.lb()
                    && lb.has_conn()
                    && score > lb.entries[9].1.into()
                    && score > 10
                {
                    do_highscore(&mut ui, score)?
                } else {
                    let game_over_text =
                        GAME_OVER_TEXT.replace("000", &format!(len 3, "{score:0>3}"));
                    let popup = Popup::new(&game_over_text).with_color(Color::Red);
                    let pos = ui.draw_centered(&popup, true)?;
                    if ui.term().wait_enter(Some(Duration::from_secs(10)))? == KeyEvent::Exit {
                        break;
                    }
                    ui.clear_centered(&popup, pos)?;
                    false
                };

                // TODO: we don't need to do this if `do_highscore` was called
                if let Some(lb) = ui.lb() {
                    lb.score = None;
                    ui.reset_lb(needs_lb_update)?;
                }
                ui.reset_stats()?;
                ui.clear_canvas()?;
            }
            None => break,
        }
    }

    Ok(())
}

fn do_highscore(ui: &mut GameUi, score: usize) -> Result<bool> {
    ui.term().clear_input()?;

    let game_over_text = ADD_LB_TEXT.replace("000", &format!(len 3, "{score:0>3}"));

    let popup = Popup::new(&game_over_text).with_color(Color::Green);
    let pos = ui.draw_centered(&popup, false)?;
    let mut colored_left = true;
    let mut next_update = Instant::now()? + Duration::from_millis(500);
    let mut cursor_pos = 0;
    let mut input = [0u8; 3];

    let ret = loop {
        match ui
            .term()
            .get_key_timeout(Some(next_update - Instant::now()?), |k| {
                matches!(k, Key::Char(_) | Key::Back | Key::Enter | Key::Esc)
            })? {
            Some(Key::Char(ch)) if cursor_pos < 3 => {
                let ch = ch.to_ascii_uppercase();
                input[cursor_pos as usize] = ch;
                cursor_pos += 1;
                ui.term().draw(
                    pos.0 + 10 + cursor_pos,
                    pos.1 + 6,
                    format!(len 9, "\x1B[1m{}\x1B[0m", ch as char).as_str(),
                )?;
            }
            Some(Key::Back) if cursor_pos > 0 => {
                ui.term()
                    .draw(pos.0 + 10 + cursor_pos, pos.1 + 6, "\x1B[2m-\x1B[0m")?;
                cursor_pos -= 1;
            }
            Some(Key::Enter) if cursor_pos == 3 => {
                ui.lb().unwrap().send_game(input, score as u8)?;
                ui.update_lb(leaderboard::LeaderboardUpdate::FillPlayer(input))?;
                break true;
            }
            Some(Key::Esc) if cursor_pos == 0 => break false,
            _ => (),
        }

        let now = Instant::now()?;
        if now > next_update {
            let str = if colored_left {
                "\x1B[32mGREAT \x1B[1mSCORE\x1B[0m"
            } else {
                "\x1B[32;1mGREAT \x1B[22mSCORE\x1B[0m"
            };
            ui.term().draw(pos.0 + 12, pos.1 + 1, str)?;
            colored_left = !colored_left;
            next_update = now + Duration::from_millis(500);
        }
    };

    ui.clear_centered(&popup, pos)?;
    Ok(ret)
}
