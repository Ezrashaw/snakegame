use std::thread;

use oca_io::{CircularBuffer, Result};
use oca_term::{Color, Key, Pixel};

use crate::{
    snake::{self, Direction},
    ui::{Coord, GameUi},
};

pub fn run(ui: &mut GameUi) -> Result<bool> {
    let mut head = snake::STARTING_POS;
    let mut tail = CircularBuffer::<Coord, { snake::STARTING_LENGTH * 2 + 1 }>::new();
    let (mut dir, mut left) = get_dir(0, &mut head).unwrap();
    let mut move_segment = 0;

    ui.draw_canvas(Coord { x: 24, y: 8 }, Pixel::new(Color::Yellow, true))?;

    loop {
        ui.draw_canvas(head, Pixel::new(Color::Green, true))?;
        tail.push(head);

        if tail.len() > snake::STARTING_LENGTH * 2 {
            ui.draw_canvas(tail.pop().unwrap(), Pixel::Clear)?;
        }

        let fruit_coord = match head {
            Coord { x: 24, y: 8 } => Some(Coord { x: 17, y: 4 }),
            Coord { x: 17, y: 4 } => Some(Coord { x: 6, y: 15 }),
            Coord { x: 6, y: 15 } => Some(Coord { x: 24, y: 8 }),
            _ => None,
        };
        if let Some(fruit) = fruit_coord {
            ui.draw_canvas(fruit, Pixel::new(Color::Yellow, true))?;
        }

        ui.flush()?;
        thread::sleep(snake::STARTING_STEP_TIME);

        ui.draw_canvas(head, Pixel::new(Color::Green, false))?;

        left -= 1;
        match dir {
            Direction::Up => head.y -= 1,
            Direction::Down => head.y += 1,
            Direction::Right => head.x += 1,
            Direction::Left => head.x -= 1,
        }

        if left == 0 {
            move_segment += 1;
            (dir, left) = get_dir(move_segment, &mut head).unwrap_or_else(|| {
                move_segment = 0;
                get_dir(move_segment, &mut head).unwrap()
            });
        }

        if ui.term().get_key(|k| k == Key::Enter)?.is_some() {
            return Ok(false);
        }

        if ui.update_tick(false)? {
            return Ok(true);
        }
    }
}

pub const fn get_dir(move_segment: u8, head: &mut Coord) -> Option<(Direction, u8)> {
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
