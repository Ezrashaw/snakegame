use std::{array, thread, time::Duration};

use crate::ui::{CANVAS_H, CANVAS_W, Coord, GameUi};

use oca_io::Result;
use oca_term::{Color, Pixel};

pub fn game_main(ui: &mut GameUi) -> Result<Option<usize>> {
    //ui.draw_canvas(Coord { x: 3, y: 3 }, Tet)?;
    let mut tetromino = Tetromino::new(Coord { x: 0, y: 0 });
    tetromino.draw(false, ui)?;

    let mut static_colors: [Option<Color>; (CANVAS_W * CANVAS_H) as usize] =
        [None; (CANVAS_W * CANVAS_H) as usize];

    loop {
        ui.term().flush()?;
        thread::sleep(Duration::from_millis(250));

        tetromino.draw(true, ui)?;
        tetromino.pos.y += 1;
        tetromino.draw(false, ui)?;

        if tetromino.pos.y > 18 {
            break;
        }
    }
    Ok(None)
}

pub enum TetrominoType {
    Box,
}

pub struct Tetromino {
    kind: TetrominoType,
    pos: Coord,
}

impl Tetromino {
    pub const fn new(pos: Coord) -> Self {
        Self {
            kind: TetrominoType::Box,
            pos,
        }
    }

    pub fn draw(&self, blank: bool, ui: &mut GameUi) -> Result<()> {
        let color = match self.kind {
            TetrominoType::Box => Color::Yellow,
        };

        let mut pp = |dx: i16, dy: i16| {
            ui.draw_canvas(
                self.pos.moved(dx, dy),
                if blank {
                    Pixel::Clear
                } else {
                    Pixel::new(color, false)
                },
            )
        };

        match self.kind {
            TetrominoType::Box => {
                pp(0, 0)?;
                pp(1, 1)?;
                pp(0, 1)?;
                pp(1, 0)?;
            }
        }

        Ok(())
    }
}
