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
pub struct FpgaIO {
    io: Box<[u8]>,
    trim: u8,
}

impl FpgaIO {
    #[inline]
    pub fn new(mut capacity: usize) -> Self {
        capacity += 2;
        let pagination = capacity / 8 + (capacity % 8 > 0) as usize;
        let mut io = Vec::with_capacity(pagination);

        for _ in 0..pagination {
            io.push(0);
        }

        Self {
            io: io.into_boxed_slice(),
            trim: ((capacity - 2) % 8) as u8,
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.io.len()
    }

    #[inline]
    fn cell_io_at(&self, cell_pos: usize) -> CellIO {
        let pagination = cell_pos / 8;
        let trim = cell_pos % 8;

        let mut bits: u8 = (self.io[pagination] >> trim) & 0b11;
        bits |= (self.io[self.len() - 1] >> 4) & 0b1100;

        CellIO::from_bits(bits).unwrap()
    }

    fn set(&mut self, cell_pos: usize, value: CellIO) {
        let pagination = cell_pos / 8;
        let trim = cell_pos % 8;

        let mut bits: u8 = value.bits();
        self.io[pagination] &= !(0b11 << trim);
        self.io[pagination] |= (bits & 0b11) << trim;
        bits = bits << 4;
        self.io[self.len() - 1] &= !(0b11 << 6);
        self.io[self.len() - 1] |= (bits & (0b11 << 2)) << 6;
    }

    fn reset_row_io(&mut self) {
        self.io[self.len() - 1] &= !(0b11 << 6);
    }
}

impl From<Box<[bool]>> for FpgaIO {
    fn from(value: Box<[bool]>) -> Self {
        let capacity = value.len() + 2;
        let pagination = capacity / 8 + (capacity % 8 > 0) as usize;
        let mut flags = vec![0u8; pagination];

        for (i, val) in value.iter().enumerate() {
            flags[i / 8] |= (*val as u8) << (i % 8);
        }

        Self {
            io: flags.into_boxed_slice(),
            trim: ((capacity - 2) % 8) as u8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FPGA {
    grid: Grid,
}

impl FPGA {
    pub fn new(grid: &Grid) -> Self {
        Self { grid: grid.clone() }
    }

    pub fn eval(&self, mut input: FpgaIO) -> Result<FpgaIO, &'static str> {
        if input.len() * 8 + input.trim as usize - 2 != self.grid.width * 2 {
            return Err("FpgaIO size does not match grid input requirements");
        }

        let mut i = 0;
        let mut j = 0;
        let mut dir: i8 = 1;

        for _ in 0..self.grid.height * (self.grid.width) {
            let cell_io = self.grid.get(j, i).unwrap().eval_cell(input.cell_io_at(i));

            input.set(i, cell_io);

            if (i == self.grid.width - 1 && dir == 1) || i == 0 && dir == -1 {
                dir *= -1;
                j += 1;
                input.reset_row_io();
            } else {
                i = (i as isize + dir as isize) as usize;
            }
        }

        Ok(input)
    }
}
