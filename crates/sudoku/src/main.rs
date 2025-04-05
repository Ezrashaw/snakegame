use oca_io::Result;
use oca_term::{Key, KeyEvent, Rect, Terminal};
use sudoku::Sudoku;

fn main() -> Result<()> {
    let mut term = Terminal::new()?;
    let (w, h) = term.size();

    let mut game = Sudoku::EMPTY;

    let (bx, by) = term.draw_centered(&game, Rect::new(1, 1, w, h))?;
    let (mut cx, mut cy) = (0, 0);

    loop {
        term.set_cursor(Some((bx + 1 + cx + cx / 3, by + 1 + cy + cy / 3)))?;

        term.flush()?;
        let key = term.wait_key(
            |k| {
                matches!(
                    k,
                    Key::Up | Key::Down | Key::Left | Key::Right | Key::Char(b'1'..=b'9')
                )
            },
            None,
        )?;

        match key {
            KeyEvent::Key(Key::Up) if cy > 0 => cy -= 1,
            KeyEvent::Key(Key::Down) if cy < 8 => cy += 1,
            KeyEvent::Key(Key::Right) if cx < 8 => cx += 1,
            KeyEvent::Key(Key::Left) if cx > 0 => cx -= 1,

            KeyEvent::Key(Key::Char(num @ b'1'..=b'9')) => {
                let idx = cy * 9 + cx;
                game.place_number(idx, num - b'0');
                term.draw_centered(&game, Rect::new(1, 1, w, h))?;
            }
            KeyEvent::Key(_) => (),
            KeyEvent::Exit => break,
            _ => unreachable!(),
        }

        // FIXME: this isn't immediate, just happens after a key is pressed, need some of that
        // signal stuff.
        if term.process_signals()? {
            return Ok(());
        }
    }

    Ok(())
}
