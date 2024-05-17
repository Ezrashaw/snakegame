use super::Color;

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
                    "BLUE" => new.push_str(Color::Blue.as_ansi()),
                    "WHITE" => new.push_str(Color::White.as_ansi()),
                    "BYELLOW" => new.push_str(Color::BrightYellow.as_ansi()),
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
