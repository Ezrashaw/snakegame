use oca_term::Color;
use std::{env, fs, path::Path, process::Command};

pub fn compile_pansi(dir: &str) {
    let cmd_output = Command::new("git")
        .args(["show", "HEAD", "--format=reference", "--no-patch"])
        .output()
        .unwrap()
        .stdout;

    let mut cmd_output = String::from_utf8(cmd_output).unwrap();
    let half_git = cmd_output.len() / 2;
    let half_git = cmd_output[half_git..].find(' ').unwrap() + half_git;
    cmd_output.replace_range(half_git..=half_git, "\n");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("git.txt");
    // note that we have to remove the trailing newline so that we only print two lines, otherwise
    // we start scrolling and other things move up a line.
    fs::write(
        dest_path,
        format!("\x1B[2m{}\x1B[0m", cmd_output.trim_matches('\n')),
    )
    .unwrap();

    for file in fs::read_dir(dir).unwrap() {
        let file = file.unwrap();
        let data = fs::read_to_string(file.path()).unwrap();
        fs::write(
            Path::new(&out_dir).join(file.file_name()),
            from_pansi(&data),
        )
        .unwrap();
    }

    for file in fs::read_dir("../pansi/global").unwrap() {
        let file = file.unwrap();
        let data = fs::read_to_string(file.path()).unwrap();
        fs::write(
            Path::new(&out_dir).join(file.file_name()),
            from_pansi(&data),
        )
        .unwrap();
    }

    println!("cargo::rerun-if-changed=.git/HEAD");
    println!("cargo::rerun-if-changed={dir}");
    println!("cargo::rerun-if-changed=../pansi/global/");
}

#[must_use]
pub fn from_pansi(s: &str) -> String {
    let s = s.trim_end_matches('\n');
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
                    "RED" => new.push_str(Color::to_str(&Color::Red.fg())),
                    "GREEN" => new.push_str(Color::to_str(&Color::Green.fg())),
                    "YELLOW" => new.push_str(Color::to_str(&Color::Yellow.fg())),
                    "BLUE" => new.push_str(Color::to_str(&Color::Blue.fg())),
                    "WHITE" => new.push_str(Color::to_str(&Color::White.fg())),

                    "BRED" => new.push_str(Color::to_str(&Color::Red.fg_bright())),
                    "BGREEN" => new.push_str(Color::to_str(&Color::Green.fg_bright())),
                    "BYELLOW" => new.push_str(Color::to_str(&Color::Yellow.fg_bright())),
                    "BCYAN" => new.push_str(Color::to_str(&Color::Cyan.fg_bright())),

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
