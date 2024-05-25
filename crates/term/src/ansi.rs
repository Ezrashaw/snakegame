use super::Color;

#[must_use]
pub fn from_pansi(s: &str) -> String {
    let mut new = String::with_capacity(s.len() * 2);
    let mut i = 0;
    loop {
        let ch = s.as_bytes()[i];
        if ch == b'{' {
            let end = s[i..].find('}').unwrap();
            let ansi = &s[(i + 1)..(i + end)];
            i += end + 1;

            new.push_str("\x1B[");
            for (idx, part) in ansi.split(';').enumerate() {
                if idx != 0 {
                    new.push(';');
                }
                match part {
                    // colors
                    "RED" => new.push_str(Color::Red.as_ansi()),
                    "GREEN" => new.push_str(Color::Green.as_ansi()),
                    "YELLOW" => new.push_str(Color::Yellow.as_ansi()),
                    "BLUE" => new.push_str(Color::Blue.as_ansi()),
                    "WHITE" => new.push_str(Color::White.as_ansi()),
                    "BYELLOW" => new.push_str(Color::BrightYellow.as_ansi()),
                    "BGREEN" => new.push_str(Color::BrightGreen.as_ansi()),
                    "BCYAN" => new.push_str(Color::BrightCyan.as_ansi()),

                    // formats
                    "RESET" => new.push('0'),
                    "BOLD" => new.push('1'),
                    "DIM" => new.push('2'),
                    "NDIM" | "NBOLD" => new.push_str("22"),
                    _ => panic!("unknown colour/formatter: {part}"),
                }
            }
            new.push('m');
        } else {
            let mut iter = s[i..].char_indices();
            new.push(iter.next().unwrap().1);
            if let Some((idx, _)) = iter.next() {
                i += idx;
            } else {
                break;
            }
        }
    }

    new.push_str("\x1B[0m");
    new
}

#[must_use]
pub fn ansi_str_len(s: &str) -> u16 {
    let mut len = 0;
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\x1B' {
            let mut ch = ch;
            while ch != 'm' {
                ch = chars.next().unwrap();
            }
        } else {
            len += 1;
        }
    }
    len
}

#[cfg(test)]
mod tests {
    use super::ansi_str_len;

    #[test]
    fn ansi_len_empty() {
        assert!(ansi_str_len("") == 0);
    }

    #[test]
    fn ansi_len_empty2() {
        assert!(ansi_str_len("\x1B[11121;424m") == 0);
    }

    #[test]
    fn ansi_len_help_text() {
        assert!(ansi_str_len("MOVE WITH \x1B[1;36mARROW KEYS\x1B[0m; EAT \x1B[1;31mFRUIT\x1B[0m; AVOID \x1B[1;32mTAIL\x1B[0m AND \x1B[1;2;37mWALLS\x1B[0m") == 53);
    }
}
