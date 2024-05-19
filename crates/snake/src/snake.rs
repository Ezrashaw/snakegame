//! Main game logic for Snake.
//!
//! This module contains the main game logic for the game. It uses the safe, high-level API exposed
//! by the other modules in this crate. I have (and continue to) endeavour to keep this module well
//! documented and easy to understand.
//!
//! The main entry point for this module is [`game_main`] which expects that the terminal UI has
//! already been setup. This function runs the game through to completion.

use std::{fs::File, io::Read, thread, time::Duration};

use oca_io::{CircularBuffer, Result};
use term::{Color, Key, Pixel};

use crate::ui::{Coord, GameUi, CANVAS_H, CANVAS_W};

/// Defines the time between each movement of the snake. Over the couse of the game, this value
/// will decrease. During this time, if a key is pressed, then we process the key event, and wait
/// for the remainer of the time.
pub const STARTING_STEP_TIME: Duration = Duration::from_millis(140);

/// Defines the starting length of the snake. Note that the snake does not actualy start at this
/// length, but slowly expands out of a single point.
pub const STARTING_LENGTH: usize = 7;

/// Defines the starting locations for the fruits. At the beginning of the game, we do not choose
/// random locations for the fruits, instead we create an 'X' pattern (from this constant).
/// Throughout the game, the number of fruits is _always_ equal to the length of this array.
pub const FOOD_LOCATIONS: [(u16, u16); 5] = [(18, 6), (18, 12), (24, 6), (24, 12), (21, 9)];

/// Defines where the snake begins on the canvas. This is defined in terms of [`CANVAS_H`], as we
/// calculate from the vertical center of the screen. Note that the snake starts here as a single
/// point and "grows" outwards from this point.
pub const STARTING_POS: Coord = Coord {
    x: 3,
    y: CANVAS_H / 2,
};

/// Main entry point for the game logic.
///
/// Returns [`None`] if the game exits because of a user action (Ctrl-C). Otherwise, returns
/// `Some(score)`.
pub fn game_main(ui: &mut GameUi) -> Result<Option<usize>> {
    // Open /dev/urandom, a fast source of entropy on Linux systems.
    let mut rng = File::open("/dev/urandom").unwrap();

    // -- snake state --
    // Initialize the snake's initial head position.
    let mut head = STARTING_POS;
    // We face towards the rest of the canvas, that is, rightwards.
    let mut direction = Direction::Right;
    // The tail starts out being empty, we add to it and trim it to keep it less than `len`.
    let mut tail = CircularBuffer::<Coord, { (CANVAS_W * CANVAS_H) as usize }>::new();
    // Initialize the bitboard that we use to determine valid locations for placing fruits. We take
    // the number of game cells, divided by size of each value (64 bits). Note also that division
    // rounds down, so we have to add another u64 (which will only be partly filled).
    let mut bitboard = [0u64; (CANVAS_W * CANVAS_H) as usize / 64 + 1];
    // Initialize the current step time to `STARTING_STEP_TIME`.
    let mut step_time = STARTING_STEP_TIME;
    // Initialize the snake's length to the starting length.
    let mut len = STARTING_LENGTH;

    // Initialize the fruits from the locations in `FOOD_LOCATIONS`.
    for (x, y) in FOOD_LOCATIONS {
        let coord = Coord { x, y };

        // Plot the fruit on the canvas.
        set_bb(&mut bitboard, coord, true);
        ui.draw_canvas(coord, Pixel::new(Color::Yellow, true))?;
    }

    loop {
        // Put the head into the tail, and mark it as occupied on the bitboard.
        tail.push(head);
        set_bb(&mut bitboard, head, true);

        // If the tail is longer than the snake's length, trim it.
        if tail.len() > len {
            let coord = tail.pop().unwrap();

            set_bb(&mut bitboard, coord, false);
            ui.draw_canvas(coord, Pixel::Clear)?;
        }

        // Draw the snake's head onto the screen.
        ui.draw_canvas(head, Pixel::new(Color::Green, true))?;

        // Sleep for the current step time, so that the snake doesn't move instantly.
        thread::sleep(step_time);

        // Check for keys, but don't wait for anything (we've already waited).
        // TODO: instead of this if-let block binding `key`, we want it to bind `direction` so we
        // don't need the gross unwrap.
        if let Some(key) = ui
            .term()
            .get_key(|k| direction.change_from_key(k).is_some())?
        {
            // Handle a movement keypress, mapping keys to their respective directions.
            direction = direction.change_from_key(key).unwrap();
        }

        // Actually move the snake's head position, checking to see if we have hit a wall.
        let old_pos = head;
        match direction {
            Direction::Up if head.y > 0 => head.y -= 1,
            Direction::Down if head.y < CANVAS_H - 1 => head.y += 1,
            Direction::Right if head.x < CANVAS_W - 1 => head.x += 1,
            Direction::Left if head.x > 0 => head.x -= 1,
            _ => break,
        }

        // Check if we have encountered *something*, we'll find out what it is below.
        if get_bb(&bitboard, head) {
            // If we have hit our own tail, then we die...
            if tail.iter().any(|h| h == head) {
                // Make sure to reset the head position back to where it was, otherwise the animation
                // mucks up.
                head = old_pos;
                break;
            }

            // ...otherwise, we have eaten a fruit.
            len += 1;

            // Speed the snake up a little.
            step_time -= Duration::from_micros(500);

            // Generate another fruit to replace that one we just ate. Note that we needn't remove
            // fruit from the bitboard because we ate it and will "digest" it (the normal snake
            // code will remove it).
            gen_fruit(&mut rng, ui, &mut bitboard)?;

            // Tell the game's UI that we have a new score, this updates the leaderboard
            // statistics panel.
            ui.update_score(len - STARTING_LENGTH)?;
        }

        // Draw the previous head position as the tail colour.
        ui.draw_canvas(old_pos, Pixel::new(Color::Green, false))?;

        // Update the game's UI, currently just the leaderboard and stats panel. This function also
        // checks for SIGINT and SIGTERM, and if one of these signals is received, then we will
        // exit here.
        if ui.update_tick(true)? {
            return Ok(None);
        }
    }

    // Do a fun little death animation.
    for coord in tail.iter().rev().skip(1) {
        ui.draw_canvas(coord, Pixel::new(Color::Red, false))?;
        thread::sleep(Duration::from_millis(50));
    }
    thread::sleep(Duration::from_millis(150));
    ui.draw_canvas(head, Pixel::new(Color::Red, true))?;
    thread::sleep(Duration::from_millis(500));

    // Return the score, calculated as the difference between the initial and current length.
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
///    increment on free squares).
/// 4. Place the fruit on the canvas.
fn gen_fruit(rng: &mut File, ui: &mut GameUi, bitboard: &mut [u64]) -> Result<()> {
    // Read eight bytes (a u64) into a buffer.
    let mut rand = [0u8; 8];
    rng.read_exact(&mut rand).unwrap();
    let rand = usize::from_le_bytes(rand);

    // Calculate how many filled and free squares there are.
    let filled = bitboard.iter().map(|x| x.count_ones()).sum::<u32>() as usize;
    let free = (CANVAS_W as usize * CANVAS_H as usize) - filled + 1;
    // Calculate our target square based on the number of free squares and our random number.
    let target_idx = rand % free;

    let mut idx = 0;
    let (mut fx, mut fy) = (u16::MAX, u16::MAX);
    // For each xy point on the canvas...
    'outer: for y in 0..CANVAS_H {
        for x in 0..CANVAS_W {
            // We only increment our index on free squares.
            if !get_bb(bitboard, Coord { x, y }) {
                idx += 1;
            }

            // If we have made it to the target index, then exit.
            if idx >= target_idx {
                fx = x;
                fy = y;
                break 'outer;
            }
        }
    }

    // Sanity check to that we did manage to find a position for the fruit. Note that this is
    // reached when the player "wins" (fills up whole canvas with the snake).
    // TODO: gracefully handle the win condition
    assert_ne!(fx, u16::MAX);

    // Mark our new fruit's location on the bitboard and draw the fruit to the screen.
    let coord = Coord { x: fx, y: fy };
    set_bb(bitboard, coord, true);
    ui.draw_canvas(coord, Pixel::new(Color::Yellow, true))?;

    Ok(())
}

/// Mark a coordinate on the bitboard as either occupied or unoccupied.
fn set_bb(bitboard: &mut [u64], coord: Coord, value: bool) {
    // Turn the 2d coordinate into a flat index.
    let idx = coord.as_idx();
    // Use magic bitwise operators to set/unset.
    if value {
        bitboard[idx / 64] |= 0b1 << (idx % 64);
    } else {
        bitboard[idx / 64] &= !(0b1 << (idx % 64));
    }
}

/// Check whether a coordinate on the bitboard is occupied or unoccupied.
const fn get_bb(bitboard: &[u64], coord: Coord) -> bool {
    // Turn the 2d coordinate into a flat index.
    let idx = coord.as_idx();
    // Use magic bitwise operators to check if the bit is marked as occupied.
    bitboard[idx / 64] & (0b1 << (idx % 64)) != 0
}

/// Enumeration representing the four possible directions that the snake can be moving in.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    /// Convert a keypress event into a direction for the snake, checking that the snake isn't
    /// doubling back on itself or continuing in the same direction (the latter improves input
    /// "feel").
    pub const fn change_from_key(self, key: Key) -> Option<Self> {
        Some(match (self, key) {
            (Self::Left | Self::Right, Key::Char(b'w')) => Self::Up,
            (Self::Left | Self::Right, Key::Char(b's')) => Self::Down,
            (Self::Up | Self::Down, Key::Right)=> Self::Right,
            (Self::Up | Self::Down, Key::Left) => Self::Left,
            _ => return None,
        })
    }
}
