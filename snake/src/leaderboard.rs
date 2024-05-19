use std::{
    io::{self, Read},
    net::{Ipv4Addr, TcpStream},
};

use crate::{terminal::Terminal, Rect};

type Entries = [([u8; 8], u8); 10];

const YOU_NAME: &str = "--\x1B[95mYOU!\x1B[90m--";

pub struct Leaderboard {
    rect: Rect,
    you_row: Option<u16>,
    you: Option<u8>,
    entries: Entries,
    conn: TcpStream,
}

impl Leaderboard {
    pub fn init(terminal: &mut Terminal, canvas: Rect) -> io::Result<Option<Self>> {
        fn init_tcp() -> io::Result<(Entries, TcpStream)> {
            let mut conn = TcpStream::connect((Ipv4Addr::LOCALHOST, 1234))?;
            let entries = Leaderboard::entries_from_stream(&mut conn)?;
            conn.set_nonblocking(true)?;

            Ok((entries, conn))
        }

        let Ok((entries, conn)) = init_tcp() else {
            return Ok(None);
        };

        let rect = Rect::new(canvas.x + canvas.w + 3, canvas.y, 17, 12);
        terminal.draw_rect_sep(rect, rect.w, rect.h, 1, Terminal::DEFAULT_CORNERS)?;
        terminal.draw_text_centered(
            rect.move_xy(1, 1).change_size(0, -11),
            "\x1B[1mLEADERBOARD\x1B[0m",
        )?;
        for i in 1..=10 {
            terminal.draw_text(rect.x + 2, rect.y + 2 + i, &format!("{i}."))?;
        }

        Ok(Some(Self {
            rect,
            you_row: None,
            you: None,
            entries,
            conn,
        }))
    }

    pub fn check_update(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        let entries = Self::entries_from_stream(&mut self.conn);
        self.entries = match entries {
            Ok(e) => e,
            Err(err) if matches!(err.kind(), io::ErrorKind::WouldBlock) => return Ok(()),
            Err(err) => return Err(err),
        };

        self.draw_values(terminal)?;

        Ok(())
    }

    fn entries_from_stream(conn: &mut TcpStream) -> io::Result<Entries> {
        let mut buf: [u8; 100] = [0u8; 10 * 10];
        conn.read_exact(&mut buf)?;

        let mut entries = [([0u8; 8], 0u8); 10];
        for (idx, entry) in buf.array_chunks::<10>().enumerate() {
            assert_eq!(entry[9], b'\n');
            entries[idx].0 = entry[0..8].try_into().unwrap();
            entries[idx].1 = entry[8];
        }

        Ok(entries)
    }

    pub fn draw_values(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        self.you_row = None;
        for i in 0..self.entries.len() {
            let (name, score, score_color) = if let Some(you) = self.you
                && self.you_row.is_none()
                && (i + 1 == self.entries.len() || you > self.entries[i].1)
            {
                self.you_row = Some(i as u16);
                (YOU_NAME, you, 95)
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

            terminal.draw_text(
                self.rect.x + 5,
                self.rect.y + 3 + i as u16,
                &format!("\x1B[1;90m{colored_name} \x1B[1;{score_color}m{score:0>3}\x1B[0m\n",),
            )?;
        }

        Ok(())
    }

    pub fn update_you(
        &mut self,
        terminal: &mut Terminal,
        new_val: u8,
        force_redraw: bool,
    ) -> io::Result<()> {
        if let Some(you_row) = self.you_row
            && !force_redraw
            && !(you_row > 0 && new_val > self.entries[you_row as usize - 1].1)
        {
            terminal.draw_text(
                self.rect.x + 14,
                self.rect.y + 3 + you_row,
                &format!("\x1B[1;95m{new_val:0>3}\x1B[0m",),
            )
        } else {
            self.you = Some(new_val);
            self.draw_values(terminal)
        }
    }
}
