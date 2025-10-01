use crate::fpga::cell::{Cell, CellIO};
use serde::{Deserialize, Serialize};

#[allow(unused)]
pub(crate) mod cell;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    width: usize,
    height: usize,
    data: Vec<Cell>,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        let init = Cell::default();

        Self {
            width,
            height,
            data: vec![init; width * height],
        }
    }

    fn get(&self, row: usize, col: usize) -> Option<&Cell> {
        if row < self.height && col < self.width {
            Some(&self.data[row * self.width + col])
        } else {
            None
        }
    }

    fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        if row < self.height && col < self.width {
            Some(&mut self.data[row * self.width + col])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct FPGA {
    grid: Grid,
}

#[derive(Debug)]
pub enum Error {
    WrongInputSize { expected: usize, got: usize },
}

impl FPGA {
    pub fn new(grid: &Grid) -> Self {
        Self { grid: grid.clone() }
    }

    pub fn eval(&self, mut input: Vec<bool>) -> Result<Vec<bool>, Error> {
        if input.len() != self.grid.width {
            return Err(Error::WrongInputSize {
                expected: self.grid.width,
                got: input.len(),
            });
        }

        input.push(false);
        input.push(false);

        let mut i = 0;
        let mut j = 0;
        let mut dir: i8 = 1;

        for _ in 0..self.grid.height * (self.grid.width) {
            let CellIO {
                column_1,
                column_2,
                row_1,
                row_2,
            } = self.grid.get(j, i).unwrap().eval_cell(CellIO {
                column_1: input[2 * i],
                column_2: input[2 * i + 1],
                row_1: input[self.grid.width - 2],
                row_2: input[self.grid.width - 1],
            });

            input[2 * i] = column_1;
            input[2 * i + 1] = column_2;
            input[self.grid.width - 2] = row_1;
            input[self.grid.width - 1] = row_2;

            if (i == self.grid.width - 1 && dir == 1) || i == 0 && dir == -1 {
                dir *= -1;
                j += 1;
                input[self.grid.width - 2] = false;
                input[self.grid.width - 1] = false;
            } else {
                i = (i as isize + dir as isize) as usize;
            }
        }
        input.pop();
        input.pop();

        Ok(input)
    }
}
