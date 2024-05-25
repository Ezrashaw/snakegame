use std::io;

use term::{ansi_str_len, from_pansi, Terminal};

const CREDITS_TEXT: &str = include_str!("../pansi/credits.txt");
const GIT_TEXT: &str = include_str!(concat!(env!("OUT_DIR"), "/git.txt"));

pub struct GameUi {}

impl GameUi {
    pub fn init(term: &mut Terminal) -> io::Result<Self> {
        let size = term::get_termsize();

        #[cfg(feature = "term_debug")]
        for i in (1..size.1).step_by(2) {
            print!("\x1B[{i};0H{i:0>2}--+");
            for x in (1..(size.0 - 15)).step_by(5) {
                if x % 10 == 1 {
                    print!("--{:0>2}+", x + 9);
                } else {
                    print!("----+");
                }
            }
            println!("\x1B[{}G{i:0>2}", size.0 - 1);
        }

        // Draw the credits text in the bottom left corner of the screen.
        term.draw(1, size.1 - 3, &*from_pansi(CREDITS_TEXT))?;

        // Draw the git commit text in the bottom right corner of the screen.
        let git_text = from_pansi(GIT_TEXT);
        let git_width = ansi_str_len(git_text.split_once('\n').unwrap().0);
        term.draw(size.0 - git_width as u16, size.1 - 1, &*git_text)?;

        Ok(Self {})
    }
}
