mod network;

use std::{env, net::TcpStream, thread::JoinHandle};

use oca_io::network::LeaderboardEntries;
use oca_io::Result;
use term::{Box, Draw, DrawCtx, Terminal};

pub struct Leaderboard {
    pub entries: LeaderboardEntries,
    pub score: Option<u8>,
    #[allow(clippy::type_complexity)]
    conn: core::result::Result<
        TcpStream,
        Option<JoinHandle<Result<(LeaderboardEntries, TcpStream)>>>,
    >,
    addr: String,
    you_row: Option<u16>,
    has_10_pos: bool,
}

impl Leaderboard {
    pub fn init(term: &mut Terminal) -> Option<Self> {
        let addr = env::var("SNAKEADDR").ok()?;
        let Ok((entries, conn)) = network::connect_tcp(&addr) else {
            term.exit_with_error("failed to connect to the leaderboard server")
        };

        Some(Self {
            entries,
            score: None,
            conn: Ok(conn),
            addr,
            you_row: None,
            has_10_pos: true,
        })
    }

    /// Redraw all the leaderboard entries, using the provided [`DrawCtx`].
    ///
    /// This function recalculates (and redraws) the position of the "YOU" row.
    ///
    /// **Note**: there is no need to clear the previous leaderboard: the leaderboard doesn't
    /// change screen-size, so we always overwrite the entire previous leaderboard.
    fn draw_entries(&mut self, ctx: &mut DrawCtx) -> Result<()> {
        // We are redrawing all the entries, so we'll re-calculate the position of the "YOU" row.
        self.you_row = None;

        let mut str = String::new();
        let mut wants_10 = true;
        for i in 0..self.entries.len() {
            // Calculate whether we are at the last leaderboard position (i.e, #10).
            let at_last_entry = i + 1 == self.entries.len();

            // We'll choose this row to be the "YOU" row iff we have not already chosen a "YOU"
            // row, and if we are at position #10 or the player's score is more than the current
            // leaderboard entry's score. We also don't want a "YOU" row if there is no score
            // display.
            let score = if let Some(score) = self.score
                && self.you_row.is_none()
                && (at_last_entry || score > self.entries[i].1)
            {
                // If the player isn't on the leaderboard (at least at position #10), then we might
                // actually be further below that (we don't know if we are position #10 or #100),
                // so get rid of the position number.
                if score <= self.entries[9].1 {
                    wants_10 = false;
                }

                // Set `self.you_row` so that succeeding iterations of this loop know we have
                // already written the "YOU" row and so that the `Draw::update` implementation on
                // this type knows where to draw.
                self.you_row = Some(i as u16);

                // Set the name to be "YOU" with bold, pink colors.
                str.push_str("\x1B[1;95mYOU");

                // Use the score that has been given as the score for the player.
                score
            } else {
                // If we have chosen a "YOU" row, then that row wasn't taken up by a regular
                // leaderboard entry, and so we want to shift the rest of the
                // down by one.
                let offset = self.you_row.map_or(0, |_| 1);

                // Find the entry from the list, using the offset.
                let entry = &self.entries[i - offset];

                // Turn the leaderboard entry into a string slice. This will never panic because
                // the leaderboard server *should* always send us ASCII.
                let name = std::str::from_utf8(&entry.0).unwrap();

                // Append the name to the entries string, using different colors for filled and
                // unfilled leaderboard positions. Note that both types of entries use bold white
                // (default color) as the score color.
                if name == "---" {
                    // For leaderboard positions that haven't been filled, use a bold, gray color.
                    str.push_str("\x1B[1;90m---\x1B[1;39m");
                } else {
                    // For leaderboard positions that have been filled, use a green color.
                    str.push_str(&format!("\x1B[22;32m{name}\x1B[1;39m"));
                }

                // Use the leaderboard entry's score.
                entry.1
            };

            // Finally, append the score and a newline. We have already set the colour of the
            // score, which is different between the "YOU" row and other rows.
            str.push_str(&format!(" {score:0>3}\n"));
        }

        // Effectuate the removal/drawing of the #10 position, making sure to only draw if
        // needed.
        if self.has_10_pos && !wants_10 {
            ctx.draw(2, 12, "..")?;
            self.has_10_pos = false;
        } else if !self.has_10_pos && wants_10 {
            // Note that we don't have to worry about ANSI state here because we buffer the
            // leaderboard to a string and write it all at once below. This means that we can
            // assume the reset state.
            ctx.draw(2, 12, "10")?;
            self.has_10_pos = true;
        }

        // Make sure to reset the ANSI state and draw the entries to the screen.
        str.push_str("\x1B[0m");
        ctx.draw(6, 3, str)
    }
}

impl Draw for &mut Leaderboard {
    fn size(&self) -> (u16, u16) {
        (15, 14)
    }

    fn draw(self, ctx: &mut term::DrawCtx) -> Result<()> {
        ctx.draw(0, 0, Box::new(13, 12).with_separator(1))?;
        ctx.draw(2, 1, "\x1B[1;34mLEADERBOARD\x1B[0m")?;
        for i in 1..=10 {
            ctx.draw(2, 2 + i, format!("{i:0>2}."))?;
        }

        self.draw_entries(ctx)?;

        Ok(())
    }

    type Update = LeaderboardUpdate;
    fn update(self, ctx: &mut DrawCtx, update: Self::Update) -> Result<()> {
        match update {
            LeaderboardUpdate::Score(score) => {
                self.score = Some(score);
                if let Some(you_row) = self.you_row {
                    if you_row == 0 || score <= self.entries[you_row as usize - 1].1 {
                        return ctx.draw(10, 3 + you_row, format!("\x1B[1;95m{score:0>3}\x1B[0m"));
                    }
                }

                self.draw_entries(ctx)
            }
            LeaderboardUpdate::Network(block, force) => {
                if let Some(entries) = self.read_leaderboard(block) {
                    self.entries = entries;
                    self.draw_entries(ctx)?;
                } else if force {
                    self.draw_entries(ctx)?;
                }

                Ok(())
            }
            LeaderboardUpdate::Redraw => self.draw_entries(ctx),
            LeaderboardUpdate::FillPlayer(name) => {
                let (you_row, score) = (self.you_row.unwrap(), self.score.unwrap());
                let name = std::str::from_utf8(&name).unwrap();
                ctx.draw(
                    6,
                    3 + you_row,
                    format!("\x1B[32m{name} \x1B[1;39m{score:0>3}\x1B[0m"),
                )
            }
        }
    }
}

pub enum LeaderboardUpdate {
    Score(u8),
    Network(bool, bool),
    Redraw,
    FillPlayer([u8; 3]),
}
