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

const STEP_MS: u64 = 140;
const STARTING_LENGTH: usize = 7;
const FOOD_COUNT: usize = 5;

pub fn game_main(mut canvas: Canvas) -> io::Result<Option<usize>> {
    let mut rng = File::open("/dev/urandom")?;

    let mut head = Coord {
        x: 3,
        y: canvas.h() / 2,
    };
    let mut direction = Direction::Right;
    let mut tail = VecDeque::new();
    let mut bitboard = vec![0u64; canvas.w() as usize * canvas.h() as usize / 64 + 1];
    let mut len = STARTING_LENGTH;
    let mut fruits: [Coord; FOOD_COUNT] =
        array::from_fn(|_| gen_fruit(&mut rng, &mut canvas, &mut bitboard).unwrap());

    loop {
        tail.push_back(head);
        let idx = head.as_idx(&canvas);
        bitboard[idx / 64] |= 0b1 << (idx % 64);
        if tail.len() > len {
            let coord = tail.pop_front().unwrap();
            canvas.clear_pixel(coord)?;

            let idx = coord.as_idx(&canvas);
            bitboard[idx / 64] &= !(0b1 << (idx % 64));
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

            // poll interupted our sleep, so we have to sleep the rest of the
            // 140ms
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

        let idx = head.as_idx(&canvas);
        if bitboard[idx / 64] & (0b1 << (idx % 64)) != 0 && !fruits.contains(&head) {
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
    // let target_idx = rand % ((canvas.w() as usize * canvas.h() as usize) - filled);
    let target_idx = rand % ((canvas.w() as usize * canvas.h() as usize) - filled + 1);

    let mut idx = 0;
    let (mut fx, mut fy) = (u16::MAX, u16::MAX);
    // for each xy point on the canvas...
    'outer: for y in 0..canvas.h() {
        for x in 0..canvas.w() {
            // get the flat index of the xy point...
            let i = Coord { x, y }.as_idx(canvas);
            // ...and check if it is occupied
            if bitboard[i / 64] & (0b1 << (i % 64)) == 0 {
                // if not, then
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
    let idx = coord.as_idx(canvas);
    bitboard[idx / 64] |= 0b1 << (idx % 64);
    canvas.draw_pixel(coord, Color::BrightYellow)?;
    Ok(coord)
}

#[derive(PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}
