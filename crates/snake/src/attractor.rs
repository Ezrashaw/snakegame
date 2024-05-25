use std::{collections::VecDeque, io, thread, time::Duration};

use term::{Color, Key, KeyEvent};

use crate::{
    snake::{self, Direction},
    ui::{Coord, GameUi},
};

pub fn run(ui: &mut GameUi) -> io::Result<bool> {
    let mut head = snake::STARTING_POS;
    let mut tail = VecDeque::with_capacity(snake::STARTING_LENGTH);
    let (mut dir, mut left) = get_dir(0, &mut head).unwrap();
    let mut move_segment = 0;

    ui.draw_pixel(Coord { x: 24, y: 7 }, Color::BrightYellow)?;

    loop {
        ui.draw_pixel(head, Color::BrightGreen)?;
        tail.push_back(head);

        if tail.len() > snake::STARTING_LENGTH * 2 {
            ui.clear_pixel(tail.pop_front().unwrap())?;
        }

        match head {
            Coord { x: 24, y: 7 } => ui.draw_pixel(Coord { x: 17, y: 3 }, Color::BrightYellow)?,
            Coord { x: 17, y: 3 } => ui.draw_pixel(Coord { x: 6, y: 14 }, Color::BrightYellow)?,
            Coord { x: 6, y: 14 } => ui.draw_pixel(Coord { x: 24, y: 7 }, Color::BrightYellow)?,
            _ => (),
        }

        thread::sleep(snake::STARTING_STEP_TIME);

        ui.draw_pixel(head, Color::Green)?;

        left -= 1;
        match dir {
            Direction::Up => head.y -= 1,
            Direction::Down => head.y += 1,
            Direction::Right => head.x += 1,
            Direction::Left => head.x -= 1,
        }

        if left == 0 {
            move_segment += 1;
            if let Some(segment) = get_dir(move_segment, &mut head) {
                (dir, left) = segment;
            } else {
                move_segment = 0;
                (dir, left) = get_dir(move_segment, &mut head).unwrap();
            }
        }

        match ui
            .term()
            .wait_key(|k| matches!(k, Key::Enter), Some(Duration::ZERO), false)?
        {
            KeyEvent::Timeout => (),
            KeyEvent::Exit => return Ok(true),
            KeyEvent::Key(_) => return Ok(false),
        }
    }
}

pub fn get_dir(move_segment: u8, head: &mut Coord) -> Option<(Direction, u8)> {
    #[allow(clippy::match_same_arms)]
    Some(match move_segment {
        0 => (Direction::Up, 4),
        1 => (Direction::Right, 22),
        2 => (Direction::Down, 8),
        3 => (Direction::Left, 2),
        4 => (Direction::Up, 7),
        5 => (Direction::Right, 1),
        6 => (Direction::Down, 10),
        7 => (Direction::Left, 13),
        8 => (Direction::Up, 3),
        9 => (Direction::Right, 3),
        10 => (Direction::Down, 1),
        11 => (Direction::Left, 2),
        12 => (Direction::Down, 1),
        13 => (Direction::Right, 5),
        14 => (Direction::Up, 4),
        15 => {
            head.y -= 5;
            (Direction::Up, 2)
        }
        16 => (Direction::Right, 6),
        17 => (Direction::Down, 5),
        18 => (Direction::Left, 3),
        19 => {
            head.x -= 14;
            (Direction::Left, 2)
        }
        20 => (Direction::Down, 9),
        21 => (Direction::Right, 1),
        22 => (Direction::Up, 4),
        23 => (Direction::Right, 1),
        24 => (Direction::Down, 4),
        25 => (Direction::Right, 1),
        26 => (Direction::Up, 5),
        27 => (Direction::Left, 4),
        28 => (Direction::Up, 4),
        _ => return None,
    })
}
