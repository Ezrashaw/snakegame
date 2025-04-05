use std::{thread, time::Duration};

use crate::ui::{Coord, GameUi};

use oca_io::Result;
use oca_term::{Color, Pixel};

pub fn game_main(ui: &mut GameUi) -> Result<Option<usize>> {
    //ui.draw_canvas(Coord { x: 3, y: 3 }, Tet)?;
    let mut tetromino = Tetromino::new(Coord { x: 1, y: 1 });

    loop {
        tetromino.draw(ui)?;
        tetromino.pos.y += 1;

        ui.term().flush()?;
        thread::sleep(Duration::from_millis(1000));

        if tetromino.pos.y > 20 {
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

    pub fn draw(&self, ui: &mut GameUi) -> Result<()> {
        match self.kind {
            TetrominoType::Box => {
                ui.draw_canvas(self.pos, Pixel::new(Color::Yellow, false))?;
                ui.draw_canvas(self.pos.moved(-1, -1), Pixel::new(Color::Yellow, false))?;
                ui.draw_canvas(self.pos.moved(0, -1), Pixel::new(Color::Yellow, false))?;
                ui.draw_canvas(self.pos.moved(-1, 0), Pixel::new(Color::Yellow, false))?;
            }
        }

        Ok(())
    }
}
