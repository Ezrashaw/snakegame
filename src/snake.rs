use std::{
    array,
    collections::VecDeque,
    fs::File,
    io::{self, Read},
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{
    leaderboard::Leaderboard,
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

/// Defines the starting locations for the fruits. At the beginning of the game, we do not choose
/// random locations for the fruits, instead we create an 'X' pattern (from this constant).
const FOOD_LOCATIONS: [(u16, u16); FOOD_COUNT] = [(20, 3), (20, 9), (26, 3), (26, 9), (23, 6)];

/// Main entry point for the game logic.
///
/// Returns [`None`] if the game exits because of a user action (Crtl-C). Otherwise, returns
/// `Some(score)`.
pub fn game_main(mut canvas: Canvas, leaderboard: &mut Leaderboard) -> io::Result<Option<usize>> {
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
    let mut fruits: [Coord; FOOD_COUNT] = array::from_fn(|i| {
        let coord = Coord {
            x: FOOD_LOCATIONS[i].0,
            y: FOOD_LOCATIONS[i].1,
        };
        set_bb(&mut bitboard, &canvas, coord, true);
        // TODO: remove `unwrap` here
        canvas.draw_pixel(coord, Color::BrightYellow).unwrap();
        coord
    });

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
        canvas.draw_pixel(head, Color::Lime)?;

        // sleep for 140ms, but wait for keys at the same time
        let time = Instant::now();
        if let Some(key) = canvas.poll_key(STEP_MS)? {
            // map keys to directions, keeping in mind that the snake can't turn immediately back
            // on itself
            match key {
                _ if let Some(dir) = direction.change_from_key(key) => direction = dir,
                Key::CrtlC => return Ok(None),
                _ => (),
            }

            // poll interupted our sleep, so we have to sleep the rest of the 140ms
            let t = STEP_MS - time.elapsed().as_millis() as u64;
            sleep(Duration::from_millis(t));
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

        // TODO: remove `fruits` array and merge this conditional with the next
        // check if we have hit our own tail
        if get_bb(&bitboard, &canvas, head) && !fruits.contains(&head) {
            // make sure to reset the head position back to where it was, otherwise the animation
            // mucks up
            head = old_pos;
            break;
        }

        // draw the previous head position as the tail colour
        canvas.draw_pixel(old_pos, Color::Green)?;

        // check if we have encountered a fruit
        if let Some(fruit_idx) = fruits.iter().position(|&f| f == head) {
            len += 1;
            // update the local player position on the leaderboard
            leaderboard.update_you(canvas.term, (len - STARTING_LENGTH) as u8)?;
            // shouldn't remove fruit from bitboard because we ate it and will "digest" it (normal
            // snake code will remove it)
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
fn gen_fruit(rng: &mut File, canvas: &mut Canvas, bitboard: &mut [u64]) -> io::Result<Coord> {
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

    Ok(coord)
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
    // TODO: needs documentation
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
