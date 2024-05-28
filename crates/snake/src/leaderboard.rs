use std::io;

use oca_io::network::LeaderboardEntries;
use term::{Box, Draw, DrawCtx};

const YOU_NAME: &str = "\x1B[95mYOU\x1B[90m";

pub struct Leaderboard {
    you_row: Option<u16>,
    pub score: u8,
    pub entries: LeaderboardEntries,
}

impl Leaderboard {
    pub const fn init(entries: LeaderboardEntries) -> Self {
        Self {
            you_row: None,
            score: 0,
            entries,
        }
    }

    fn draw_entries(&mut self, ctx: &mut DrawCtx) -> io::Result<()> {
        self.you_row = None;
        for i in 0..self.entries.len() {
            let (name, score, score_color) = if self.you_row.is_none()
                && (i + 1 == self.entries.len() || self.score > self.entries[i].1)
            {
                self.you_row = Some(i as u16);
                (YOU_NAME, self.score, 95)
            } else {
                let offset = self.you_row.map_or(0, |_| 1);
                let entry = &self.entries[i - offset];
                let name = std::str::from_utf8(&entry.0).unwrap();
                (name, entry.1, 39)
            };

            // quicker check to see if name isn't YOU_NAME
            let colored_name = if score_color == 39 {
                let mut colored_name = String::new();
                let mut in_dashes = true;
                for ch in name.chars() {
                    if ch == '-' && !in_dashes {
                        in_dashes = true;
                        colored_name.push_str("\x1B[1;90m");
                    } else if ch != '-' && in_dashes {
                        in_dashes = false;
                        colored_name.push_str("\x1B[22;32m");
                    }
                    colored_name.push(ch);
                }

                colored_name
            } else {
                YOU_NAME.to_owned()
            };

            ctx.draw(
                6,
                3 + i as u16,
                &*format!("\x1B[1;90m{colored_name} \x1B[1;{score_color}m{score:0>3}\x1B[0m",),
            )?;
        }

        Ok(())
    }
}

impl Draw for &mut Leaderboard {
    fn size(&self) -> (u16, u16) {
        (15, 14)
    }

    fn draw(self, ctx: &mut term::DrawCtx) -> io::Result<()> {
        ctx.draw(0, 0, Box::new(13, 12).with_separator(1))?;
        ctx.draw(2, 1, "\x1B[1mLEADERBOARD\x1B[0m")?;
        for i in 1..=10 {
            ctx.draw(2, 2 + i, format!("{i:0>2}."))?;
        }

        self.draw_entries(ctx)?;

        Ok(())
    }

    type Update = LeaderboardUpdate;
    fn update(self, ctx: &mut DrawCtx, update: Self::Update) -> io::Result<()> {
        match update {
            LeaderboardUpdate::Score(score)
                if let Some(you_row) = self.you_row
                    && !(you_row > 0 && score > self.entries[you_row as usize - 1].1) =>
            {
                self.score = score;
                ctx.draw(10, 3 + you_row, format!("\x1B[1;95m{score:0>3}\x1B[0m"))
            }
            LeaderboardUpdate::Score(score) => {
                self.score = score;
                self.draw_entries(ctx)
            }
            LeaderboardUpdate::Redraw => self.draw_entries(ctx),
        }
    }
}

pub enum LeaderboardUpdate {
    Score(u8),
    Redraw,
}
