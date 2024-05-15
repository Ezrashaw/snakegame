use std::{
    array,
    collections::VecDeque,
    fs::File,
    io::{self, Read},
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{
    terminal::{Color, Key},
    Canvas, Coord,
};

/// Defines the time (in milliseconds) between each movement of the snake. During this time, if a
/// key is pressed, then we process the key event, and wait for the remainer of the time
const STEP_MS: u64 = 140;
/// Defines the starting length of the snake. Note that the snake does not actualy start at this
/// length, but slowly expands out of a single point.
const STARTING_LENGTH: usize = 7;
/// Defines the number of fruits on the canvas. Throughout the game, this is the number of fruits
/// is _always_ equal to this value.
const FOOD_COUNT: usize = 5;

/// Main entry point for the game logic.
///
/// Returns [`None`] if the game exits because of a user action (Crtl-C). Otherwise, returns
/// `Some(score)`.
pub fn game_main(mut canvas: Canvas) -> io::Result<Option<usize>> {
    // open /dev/urandom, a fast source of entropy on Linux systems.
    let mut rng = File::open("/dev/urandom")?;

    // -- snake state --
    // the snake begins in the vertical center of the screen and x = 3
    let mut head = Coord {
        x: 3,
        y: canvas.h() / 2,
    };
    // we face towards the rest of the canvas, that is, rightwards
    let mut direction = Direction::Right;
    // the tail starts out being
    let mut tail = VecDeque::new();
    // initialize the bitboard that we use to determine valid locations for placing fruits. we take
    // the number of game cells, divided by size of each value (64 bits). note also that division
    // rounds down, so we have to add another u64 (which will only be partly filled).
    let mut bitboard = vec![0u64; canvas.w() as usize * canvas.h() as usize / 64 + 1];
    // initialize the snake's length to the starting length
    let mut len = STARTING_LENGTH;
    // initialize the fruits by choosing random locations on the canvas,
    // FIXME: we haven't yet init'ed the bitboard with the snake's initial position, so we could
    // spawn a fruit and then override it by spawning the snake.
    // FIXME: don't unwrap the result of the `gen_fruit` function
    let mut fruits: [Coord; FOOD_COUNT] =
        array::from_fn(|_| gen_fruit(&mut rng, &mut canvas, &mut bitboard).unwrap());

    loop {
        tail.push_back(head);
        set_bb(&mut bitboard, &canvas, head, true);
        if tail.len() > len {
            let coord = tail.pop_front().unwrap();
            canvas.clear_pixel(coord)?;

            set_bb(&mut bitboard, &canvas, coord, false);
        }

        canvas.draw_pixel(head, Color::Lime)?;

        let time = Instant::now();
        if let Some(key) = canvas.poll_key(STEP_MS)? {
            match key {
                Key::Up if direction != Direction::Down => direction = Direction::Up,
                Key::Down if direction != Direction::Up => direction = Direction::Down,
                Key::Right if direction != Direction::Left => direction = Direction::Right,
                Key::Left if direction != Direction::Right => direction = Direction::Left,
                Key::CrtlC => return Ok(None),
                _ => (),
            }

            // poll interupted our sleep, so we have to sleep the rest of the 140ms
            let t = Instant::now() - time;
            let t = STEP_MS - t.as_millis() as u64;
            sleep(Duration::from_millis(t));
        }

        let old_pos = head;
        match direction {
            Direction::Up if head.y > 0 => head.y -= 1,
            Direction::Down if head.y < canvas.h() - 1 => head.y += 1,
            Direction::Right if head.x < canvas.w() - 1 => head.x += 1,
            Direction::Left if head.x > 0 => head.x -= 1,
            _ => break,
        }

        if get_bb(&bitboard, &canvas, head) && !fruits.contains(&head) {
            // make sure to reset the head position back to where it was,
            // otherwise the animation mucks up
            head = old_pos;
            break;
        }

        canvas.draw_pixel(old_pos, Color::Green)?;

        if let Some(fruit_idx) = fruits.iter().position(|&f| f == head) {
            len += 1;
            // shouldn't remove fruit from bitboard because we ate it and will
            // "digest" it (normal snake code will remove it)
            fruits[fruit_idx] = gen_fruit(&mut rng, &mut canvas, &mut bitboard)?;
        }
    }

    // do a fun little death animation
    for coord in tail.iter().rev().skip(1) {
        canvas.draw_pixel(*coord, Color::Red)?;
        sleep(Duration::from_millis(50));
    }
    sleep(Duration::from_millis(150));
    canvas.draw_pixel(head, Color::BrightRed)?;
    sleep(Duration::from_millis(500));

    Ok(Some(len - STARTING_LENGTH))
}

fn gen_fruit(rng: &mut File, canvas: &mut Canvas, bitboard: &mut [u64]) -> io::Result<Coord> {
    let mut idx = [0u8; 8];
    rng.read_exact(&mut idx)?;
    let rand = usize::from_le_bytes(idx);

    let filled = bitboard.iter().map(|x| x.count_ones()).sum::<u32>() as usize;
    let target_idx = rand % ((canvas.w() as usize * canvas.h() as usize) - filled + 1);

    let mut idx = 0;
    let (mut fx, mut fy) = (u16::MAX, u16::MAX);
    // for each xy point on the canvas...
    'outer: for y in 0..canvas.h() {
        for x in 0..canvas.w() {
            if !get_bb(bitboard, canvas, Coord { x, y }) {
                idx += 1;
            }
            if idx >= target_idx {
                fx = x;
                fy = y;
                break 'outer;
            }
        }
    }

    assert_ne!(fx, u16::MAX);

    let coord = Coord { x: fx, y: fy };
    set_bb(bitboard, canvas, coord, true);
    canvas.draw_pixel(coord, Color::BrightYellow)?;
    Ok(coord)
}

fn set_bb(bitboard: &mut [u64], canvas: &Canvas, coord: Coord, value: bool) {
    // TODO: inline `as_idx`
    let idx = coord.as_idx(canvas);
    if value {
        bitboard[idx / 64] |= 0b1 << (idx % 64);
    } else {
        bitboard[idx / 64] &= !(0b1 << (idx % 64));
    }
}

fn get_bb(bitboard: &[u64], canvas: &Canvas, coord: Coord) -> bool {
    // TODO: inline `as_idx`
    let idx = coord.as_idx(canvas);
    bitboard[idx / 64] & (0b1 << (idx % 64)) != 0
}

#[derive(PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}
