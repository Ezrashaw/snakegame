use std::io;

use crate::{terminal::Terminal, Rect};

pub const DEFAULT_LEADERBOARD: [([u8; 8], u8); 10] = [
    (*b"1-------", 0),
    (*b"2-------", 0),
    (*b"3-------", 0),
    (*b"4-------", 0),
    (*b"5-------", 0),
    (*b"6-------", 0),
    (*b"7-------", 0),
    (*b"8-------", 0),
    (*b"9-------", 0),
    (*b"10------", 0),
];

const YOU_NAME: &str = "--\x1B[95mYOU!\x1B[90m--";

pub struct Leaderboard {
    rect: Rect,
    you_row: Option<u16>,
    entries: [([u8; 8], u8); 10],
}

impl Leaderboard {
    pub fn init(terminal: &mut Terminal, canvas: Rect) -> io::Result<Self> {
        let rect = Rect::new(canvas.x + canvas.w + 5, canvas.y, 17, 12);
        terminal.draw_rect_sep(rect, rect.w, rect.h, 1)?;
        terminal.draw_text_centered(
            rect.move_xy(1, 1).change_size(0, -11),
            "\x1B[1mLEADERBOARD\x1B[0m",
        )?;
        for i in 1..=10 {
            terminal.draw_text(rect.x + 2, rect.y + 2 + i, &format!("{i}."))?;
        }

        Ok(Self {
            rect,
            you_row: None,
            entries: DEFAULT_LEADERBOARD,
        })
    }

    pub fn draw_values(&mut self, terminal: &mut Terminal, you: u8) -> io::Result<()> {
        self.you_row = None;
        let mut i = 1;
        while i <= self.entries.len() {
            let (name, score) = if self.you_row.is_some() {
                self.entries[i - 2]
            } else {
                self.entries[i - 1]
            };

            let (name, score, score_color) =
                if self.you_row.is_none() && (you > score || i == self.entries.len()) {
                    self.you_row = Some(i as u16);
                    (YOU_NAME, you, 95)
                } else {
                    let name = std::str::from_utf8(&name).unwrap();
                    (name, score, 39)
                };

            terminal.draw_text(
                self.rect.x + 5,
                self.rect.y + 2 + i as u16,
                &format!("\x1B[1;90m{name} \x1B[{score_color}m{score:0>3}\x1B[0m",),
            )?;

            i += 1;
        }

        Ok(())
    }

    pub fn update_you(&mut self, terminal: &mut Terminal, new_val: u8) -> io::Result<()> {
        let you_row = self.you_row.unwrap();
        if you_row > 1 {
            let next_highest = self.entries[you_row as usize - 1].1;
            if new_val > next_highest {
                self.draw_values(terminal, new_val)?;
                return Ok(());
            }
        }
        terminal.draw_text(
            self.rect.x + 14,
            self.rect.y + 2 + you_row,
            &format!("\x1B[1;95m{new_val:0>3}\x1B[0m",),
        )
    }
}
