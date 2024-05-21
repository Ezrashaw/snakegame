use std::io;

use term::*;

fn main() -> io::Result<()> {
    let mut term = Terminal::new().unwrap();

    let size = term::get_termsize();
    for i in (1..size.1).step_by(2) {
        print!("\x1B[{i};0H{i:0>2}--+");
        for x in (1..(size.0 - 20)).step_by(5) {
            if x % 10 == 1 {
                print!("--{:0>2}+", x + 9);
            } else {
                print!("----+");
            }
        }
        println!("\x1B[{}G{i:0>2}", size.0 - 1);
    }

    term.wait_key(|k| matches!(k, Key::Enter), None, true)?;

    Ok(())
}
