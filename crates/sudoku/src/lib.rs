use core::fmt;

use oca_io::Result;
use oca_term::{draw, drawln, Box, Draw, DrawCtx};

pub struct Sudoku {
    numbers: [u8; 81],
    solution: [u8; 81],
}

impl Sudoku {
    pub fn new(sol: [u8; 81]) -> Self {
        todo!()
    }

    pub const EMPTY: Self = Self {
        numbers: [0; 81],
        solution: [0; 81],
    };

    pub fn place_number(&mut self, idx: u16, num: u8) {
        assert!(num > 0 && num < 10);
        self.numbers[Into::<usize>::into(idx)] = num;
    }
}

impl Draw for &Sudoku {
    fn size(&self) -> (u16, u16) {
        (11, 11)
    }

    fn draw<W: fmt::Write>(self, ctx: &mut DrawCtx<W>) -> Result<()> {
        fn draw_num<W: fmt::Write>(ctx: &mut DrawCtx<W>, n: u8) -> Result<()> {
            if n == 0 {
                draw!(ctx, " ")?;
                return Ok(());
            }

            draw!(ctx, "{}\x1B[0m", n)?;
            Ok(())
        }

        let box_ = Box::new_tuple(self.size())
            .with_horz_lines(&[3, -3])
            .with_vert_lines(&[3, -3]);
        ctx.draw(0, 0, box_)?;
        ctx.goto(0, 1)?;

        for (lidx, line) in self.numbers.chunks_exact(9).enumerate() {
            for group in line.chunks_exact(3) {
                draw!(ctx, "\x1B[1C")?;
                draw_num(ctx, group[0])?;
                draw_num(ctx, group[1])?;
                draw_num(ctx, group[2])?;
            }

            drawln!(ctx)?;
            if (lidx + 1) % 3 == 0 {
                drawln!(ctx)?;
            }
        }

        Ok(())
    }
}
