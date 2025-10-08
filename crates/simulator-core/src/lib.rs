use crate::cell::{Cell, CellIO};
use serde::{Deserialize, Serialize};

#[allow(unused)]
pub mod cell;
pub mod macros;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FPGA {
    width: usize,
    height: usize,
    data: Vec<Cell>,
}

impl FPGA {
    #[inline]
    pub fn new(width: usize, height: usize) -> Self {
        let init = Cell::default();

        Self {
            width,
            height,
            data: vec![init; width * height],
        }
    }

    #[inline]
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&Cell> {
        if row < self.height && col < self.width {
            Some(&self.data[row * self.width + col])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        if row < self.height && col < self.width {
            Some(&mut self.data[row * self.width + col])
        } else {
            None
        }
    }

    #[inline]
    pub fn eval(&self, mut input: FpgaIO) -> Result<FpgaIO, &'static str> {
        if input.len() * 8 + input.trim as usize - 2 != self.width * 2 {
            return Err("FpgaIO size does not match grid input requirements");
        }

        let mut i = 0;
        let mut j = 0;
        let mut dir: i8 = 1;

        for _ in 0..self.height * (self.width) {
            let cell_io = self.get_cell(j, i).unwrap().eval_cell(input.cell_io_at(i));

            input.set(i, cell_io);

            if (i == self.width - 1 && dir == 1) || i == 0 && dir == -1 {
                dir *= -1;
                j += 1;
                input.reset_row_io();
            } else {
                i = (i as isize + dir as isize) as usize;
            }
        }

        Ok(input)
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
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

    #[inline]
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

    #[inline]
    fn reset_row_io(&mut self) {
        self.io[self.len() - 1] &= !(0b11 << 6);
    }
}

impl From<Box<[bool]>> for FpgaIO {
    #[inline]
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
