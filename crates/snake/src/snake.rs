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
    thread,
    time::Duration,
};

use term::{Color, Key, KeyEvent};

use crate::ui::{Coord, GameUi, CANVAS_H, CANVAS_W};

/// Defines the time between each movement of the snake. Over the couse of the game, this value
/// will decrease. During this time, if a key is pressed, then we process the key event, and wait
/// for the remainer of the time.
const STARTING_STEP_TIME: Duration = Duration::from_millis(140);

/// Defines the starting length of the snake. Note that the snake does not actualy start at this
/// length, but slowly expands out of a single point.
const STARTING_LENGTH: usize = 7;

/// Defines the starting locations for the fruits. At the beginning of the game, we do not choose
/// random locations for the fruits, instead we create an 'X' pattern (from this constant).
/// Throughout the game, the number of fruits is _always_ equal to the length of this array.
const FOOD_LOCATIONS: [(u16, u16); 5] = [(18, 5), (18, 11), (24, 5), (24, 11), (21, 8)];

/// Main entry point for the game logic.
///
/// Returns [`None`] if the game exits because of a user action (Ctrl-C). Otherwise, returns
/// `Some(score)`.
pub fn game_main(ui: &mut GameUi) -> io::Result<Option<usize>> {
    // Open /dev/urandom, a fast source of entropy on Linux systems.
    let mut rng = File::open("/dev/urandom")?;

    // -- snake state --
    // The snake begins in the vertical center of the screen and x = 3.
    let mut head = Coord {
        x: 3,
        y: CANVAS_H / 2 - 1,
    };
    // We face towards the rest of the canvas, that is, rightwards.
    let mut direction = Direction::Right;
    // The tail starts out being empty, we add to it and trim it to keep it less than `len`.
    let mut tail = VecDeque::new();
    // Initialize the bitboard that we use to determine valid locations for placing fruits. We take
    // the number of game cells, divided by size of each value (64 bits). Note also that division
    // rounds down, so we have to add another u64 (which will only be partly filled).
    let mut bitboard = vec![0u64; CANVAS_W as usize * CANVAS_H as usize / 64 + 1];
    // Initalize another bitboard that describes where the special (pink) fruits are.
    let mut special_fruits = vec![0u64; CANVAS_W as usize * CANVAS_H as usize / 64 + 1];
    // Keep track of how much "special time" is remaining. We don't start out with any special
    // time.
    let mut special_time = 0;
    // Initialize the current step time to `STARTING_STEP_TIME`.
    let mut step_time = STARTING_STEP_TIME;
    // Initialize the snake's length to the starting length.
    let mut len = STARTING_LENGTH;
    // Initialize the fruits from the locations in FOOD_LOCATIONS.
    for (x, y) in FOOD_LOCATIONS {
        let coord = Coord { x, y };

        // Plot the fruit on the canvas.
        set_bb(&mut bitboard, coord, true);
        ui.draw_pixel(coord, Color::BrightYellow)?;
    }

    loop {
        // Put the head into the tail, and mark it as occupied on the bitboard.
        tail.push_back(head);
        set_bb(&mut bitboard, head, true);

        // If the tail is longer than the snake's length, trim it.
        if tail.len() > len {
            let coord = tail.pop_front().unwrap();

            ui.clear_pixel(coord)?;
            set_bb(&mut bitboard, coord, false);
        }

        // Draw the snake's head onto the canvas.
        let head_color = if special_time > 0 {
            // If we have eaten a special fruit, then decrement the "special time" counter...
            special_time -= 1;

            // ...and use the pink color.
            Color::Magenta
        } else {
            // Otherwise, use the normal bright green color.
            Color::BrightGreen
        };
        ui.draw_pixel(head, head_color)?;

        // Sleep for the current step time, so that the snake doesn't move instantly.
        thread::sleep(step_time);

        // Check for keys, but don't wait for anything (we've already waited).
        match ui.term().wait_key(
            |k| direction.change_from_key(k).is_some(),
            Some(Duration::ZERO),
            false,
        )? {
            // If we didn't get a key, do nothing.
            KeyEvent::Timeout => (),

            // If CTRL-C is pushed, then exit.
            KeyEvent::Exit => return Ok(None),

            // Otherwise, handle a movement keypress.
            KeyEvent::Key(key) => {
                // Map movement keys to their respective directions.
                direction = direction.change_from_key(key).unwrap();
            }
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
            if tail.contains(&head) {
                // Make sure to reset the head position back to where it was, otherwise the animation
                // mucks up.
                head = old_pos;
                break;
            }

            // If we have hit a special fruit, then do something special...
            if get_bb(&special_fruits, head) {
                // Remove the special fruit from the bitboard.
                set_bb(&mut special_fruits, head, false);

                // We'll update the "special time" counter to turn the head pink for a bit.
                special_time += 15;
            }

            // ...otherwise, we have eaten a fruit.
            len += 1;

            // Every 10 fruits, speed up a little.
            if len % 10 == 0 {
                step_time -= Duration::from_millis(5);
            }

            // Update the local player position on the leaderboard.
            // if let Some(leaderboard) = leaderboard {
            //     leaderboard.update_you(canvas.term, (len - STARTING_LENGTH) as u8, false)?;
            // }

            // Needn't remove fruit from bitboard because we ate it and will "digest" it (normal
            // snake code will remove it).
            gen_fruit(&mut rng, ui, &mut bitboard, &mut special_fruits)?;
        }

        // Draw the previous head position as the tail colour.
        ui.draw_pixel(old_pos, Color::Green)?;

        // Give the leaderboard a chance to update, if we have received a new leaderboard from the
        // server.
        // if let Some(leaderboard) = leaderboard {
        //     leaderboard.check_update(canvas.term)?;
        // }

        // Update the statistics panel.
        // stats.update(&mut canvas, len - STARTING_LENGTH)?;
    }

    // Do a fun little death animation.
    for coord in tail.iter().rev().skip(1) {
        ui.draw_pixel(*coord, Color::Red)?;
        thread::sleep(Duration::from_millis(50));
    }
    thread::sleep(Duration::from_millis(150));
    ui.draw_pixel(head, Color::BrightRed)?;
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
/// increment on free squares).
/// 4. Place the fruit on the canvas.
fn gen_fruit(
    rng: &mut File,
    ui: &mut GameUi,
    bitboard: &mut [u64],
    special_fruits: &mut [u64],
) -> io::Result<()> {
    // Read eight bytes (a u64) into a buffer.
    let mut rand = [0u8; 8];
    rng.read_exact(&mut rand)?;
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

    // Mark our new fruit's location on the bitboard.
    let coord = Coord { x: fx, y: fy };
    set_bb(bitboard, coord, true);

    // Generate another random number.
    let mut rand = [0u8; 2];
    rng.read_exact(&mut rand)?;
    let rand = u16::from_le_bytes(rand);

    // There is a one in ten chance of generating a special fruit.
    let color = if rand % 10 == 0 {
        // For special fruits, add them to the special bitboard...
        set_bb(special_fruits, coord, true);

        // ...and color them pink.
        Color::BrightMagenta
    } else {
        // For normal fruits, color them yellow.
        Color::BrightYellow
    };

    // Finally, draw the fruit to the screen.
    ui.draw_pixel(coord, color)?;

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
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    /// Convert a keypress event into a direction for the snake, checking that the snake isn't
    /// doubling back on itself or continuing in the same direction (the latter improves input
    /// "feel")
    pub const fn change_from_key(self, key: Key) -> Option<Self> {
        Some(match (self, key) {
            (Self::Left | Self::Right, Key::Up | Key::Char(b'w')) => Self::Up,
            (Self::Left | Self::Right, Key::Down | Key::Char(b's')) => Self::Down,
            (Self::Up | Self::Down, Key::Right | Key::Char(b'd')) => Self::Right,
            (Self::Up | Self::Down, Key::Left | Key::Char(b'a')) => Self::Left,
            _ => return None,
        })
    }
}
