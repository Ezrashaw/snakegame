//! Main game logic for Snake.
//!
//! This module contains the main game logic for the game. It uses the safe, high-level API exposed
//! by the other modules in this crate. I have (and continue to) endeavour to keep this module well
//! documented and easy to understand.
//!
//! The main entry point for this module is [`game_main`] which expects that the terminal UI has
//! already been setup. This function runs the game through to completion.

use std::{
    collections::VecDeque,
    fs::File,
    io::{self, Read},
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{
    leaderboard::Leaderboard,
    terminal::{Color, Key, KeyEvent},
    Canvas, Coord,
};

/// Defines the time (in milliseconds) between each movement of the snake. During this time, if a
/// key is pressed, then we process the key event, and wait for the remainer of the time
const STEP_MS: u64 = 140;

/// Defines the starting length of the snake. Note that the snake does not actualy start at this
/// length, but slowly expands out of a single point.
const STARTING_LENGTH: usize = 7;

/// Defines the starting locations for the fruits. At the beginning of the game, we do not choose
/// random locations for the fruits, instead we create an 'X' pattern (from this constant).
/// Throughout the game, the number of fruits is _always_ equal to the length of this array.
const FOOD_LOCATIONS: [(u16, u16); 5] = [(18, 5), (18, 11), (24, 5), (24, 11), (21, 8)];

/// Main entry point for the game logic.
///
/// Returns [`None`] if the game exits because of a user action (Crtl-C). Otherwise, returns
/// `Some(score)`.
pub fn game_main(
    mut canvas: Canvas,
    leaderboard: &mut Option<Leaderboard>,
) -> io::Result<Option<usize>> {
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
    // initialize the fruits from the locations in FOOD_LOCATIONS
    for (x, y) in FOOD_LOCATIONS {
        let coord = Coord { x, y };

        // plot the fruit on the canvas
        set_bb(&mut bitboard, &canvas, coord, true);
        canvas.draw_pixel(coord, Color::BrightYellow)?;
    }

    loop {
        // put the head into the tail, and mark it as occupied on the bitboard
        tail.push_back(head);
        set_bb(&mut bitboard, &canvas, head, true);

        // if the tail is longer than the snake's length, trim it
        if tail.len() > len {
            let coord = tail.pop_front().unwrap();

            canvas.clear_pixel(coord)?;
            set_bb(&mut bitboard, &canvas, coord, false);
        }

        // draw the snake's head onto the canvas
        canvas.draw_pixel(head, Color::BrightGreen)?;

        // sleep for 140ms, but wait for keys at the same time; we wait only for the directional
        // keys
        let time = Instant::now();
        match canvas.wait_key(|k| direction.change_from_key(k).is_some(), Some(STEP_MS))? {
            // if we didn't get a key, do nothing
            KeyEvent::Timeout => (),

            // if CTRL-C is pushed, then exit
            KeyEvent::Exit => return Ok(None),

            // otherwise, handle a movement keypress
            KeyEvent::Key(key) => {
                // map movement keys to their respective directions
                let Some(dir) = direction.change_from_key(key) else {
                    unreachable!();
                };

                // set the new direction
                direction = dir;

                // our sleep was interrupted, so we have to sleep the rest of the time
                let t = STEP_MS - time.elapsed().as_millis() as u64;
                sleep(Duration::from_millis(t));
            }
        }

        // actually move the snake's head position, checking to see if we have hit a wall
        let old_pos = head;
        match direction {
            Direction::Up if head.y > 0 => head.y -= 1,
            Direction::Down if head.y < canvas.h() - 1 => head.y += 1,
            Direction::Right if head.x < canvas.w() - 1 => head.x += 1,
            Direction::Left if head.x > 0 => head.x -= 1,
            _ => break,
        }

        // check if we have encountered *something*, we'll find out what it is below
        if get_bb(&bitboard, &canvas, head) {
            // if we have hit our own tail, then we die...
            if tail.contains(&head) {
                // make sure to reset the head position back to where it was, otherwise the animation
                // mucks up
                head = old_pos;
                break;
            }

            // ...otherwise, we have eaten a fruit
            len += 1;

            // update the local player position on the leaderboard
            if let Some(leaderboard) = leaderboard {
                leaderboard.update_you(canvas.term, (len - STARTING_LENGTH) as u8)?;
            }

            // needn't remove fruit from bitboard because we ate it and will "digest" it (normal
            // snake code will remove it)
            gen_fruit(&mut rng, &mut canvas, &mut bitboard)?;
        }

        // draw the previous head position as the tail colour
        canvas.draw_pixel(old_pos, Color::Green)?;

        // give the leaderboard a chance to update, if we have received a new leaderboard from the
        // server
        if let Some(leaderboard) = leaderboard {
            leaderboard.check_update(canvas.term)?;
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

    // return the score, calculated as the difference between the initial and current length
    Ok(Some(len - STARTING_LENGTH))
}

/// Creates a fruit at a random position on the canvas, accounting for other fruits and the snake.
///
/// The naive approach to generating fruits is simple:
/// 1. Choose a random location.
/// 2. If it is not free, then repeat 1.
/// 3. Place the fruit on the canvas.
///
/// This approach has a significant issue. When there are few free squares left (that is, when the
/// snake is very long), a possibly infinite number of random numbers might be generated. In other
/// words, there is no fixed upper time bound for this algorithm.
///
/// Instead, a better algorithm is used that is O(n) on the size of the canvas:
/// 1. Calculate the number of free squares.
/// 2. Generate a random number between 0 and the number of free squares.
/// 3. Map the generated index onto the canvas (we iterate over the whole canvas, and only
/// increment on free squares).
/// 4. Place the fruit on the canvas.
fn gen_fruit(rng: &mut File, canvas: &mut Canvas, bitboard: &mut [u64]) -> io::Result<()> {
    // read eight bytes (a u64) into a buffer
    let mut rand = [0u8; 8];
    rng.read_exact(&mut rand)?;
    let rand = usize::from_le_bytes(rand);

    // calculate how many filled and free squares there are
    let filled = bitboard.iter().map(|x| x.count_ones()).sum::<u32>() as usize;
    let free = (canvas.w() as usize * canvas.h() as usize) - filled + 1;
    // calculate our target square based on the number of free squares and our random number
    let target_idx = rand % free;

    let mut idx = 0;
    let (mut fx, mut fy) = (u16::MAX, u16::MAX);
    // for each xy point on the canvas...
    'outer: for y in 0..canvas.h() {
        for x in 0..canvas.w() {
            // we only increment our index on free squares
            if !get_bb(bitboard, canvas, Coord { x, y }) {
                idx += 1;
            }

            // if we have made it to the target index, then exit
            if idx >= target_idx {
                fx = x;
                fy = y;
                break 'outer;
            }
        }
    }

    // sanity check to that we did manage to find a position for the fruit. note that this is
    // reached when the player "wins" (fills up whole canvas with the snake).
    // TODO: gracefully handle the win condition
    assert_ne!(fx, u16::MAX);

    // mark our new fruit's location on the bitboard and draw it to the screen
    let coord = Coord { x: fx, y: fy };
    set_bb(bitboard, canvas, coord, true);
    canvas.draw_pixel(coord, Color::BrightYellow)?;

    Ok(())
}

/// Mark a coordinate on the bitboard as either occupied or unoccupied.
fn set_bb(bitboard: &mut [u64], canvas: &Canvas, coord: Coord, value: bool) {
    // turn the 2d coordinate into a flat index
    // TODO: inline `as_idx`
    let idx = coord.as_idx(canvas);
    // use magic bitwise operators to set/unset
    if value {
        bitboard[idx / 64] |= 0b1 << (idx % 64);
    } else {
        bitboard[idx / 64] &= !(0b1 << (idx % 64));
    }
}

/// Check whether a coordinate on the bitboard is occupied or unoccupied.
const fn get_bb(bitboard: &[u64], canvas: &Canvas, coord: Coord) -> bool {
    // turn the 2d coordinate into a flat index
    // TODO: inline `as_idx`
    let idx = coord.as_idx(canvas);
    // use magic bitwise operators to check if the bit is marked as occupied
    bitboard[idx / 64] & (0b1 << (idx % 64)) != 0
}

/// Enumeration representing the four possible directions that the snake can be moving in.
#[derive(PartialEq, Eq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    /// Convert a keypress event into a direction for the snake, checking that the snake isn't
    /// doubling back on itself.
    pub fn change_from_key(self, key: Key) -> Option<Self> {
        Some(match key {
            Key::Up | Key::Char(b'w') if self != Self::Down => Self::Up,
            Key::Down | Key::Char(b's') if self != Self::Up => Self::Down,
            Key::Right | Key::Char(b'd') if self != Self::Left => Self::Right,
            Key::Left | Key::Char(b'a') if self != Self::Right => Self::Left,
            _ => return None,
        })
    }
}
