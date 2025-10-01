use bitflags::{Flags, bitflags};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[repr(u8)]
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Selector {
    Column1 = 0,
    Column2 = 1,
    Row1 = 2,
    Row2 = 3,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct ActivationOrder([Selector; 4]);

impl Default for ActivationOrder {
    fn default() -> Self {
        Self([
            Selector::Column1,
            Selector::Column2,
            Selector::Row1,
            Selector::Row2,
        ])
    }
}

impl ActivationOrder {
    pub(crate) fn new(order: [Selector; 4]) -> Result<Self, &'static str> {
        let set: HashSet<_> = order.iter().collect();
        if set.len() != 4 {
            return Err("Duplicate enum variants not allowed");
        }
        Ok(ActivationOrder(order))
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct CellIO {
    pub column_1: bool,
    pub column_2: bool,
    pub row_1: bool,
    pub row_2: bool,
}

bitflags! {
     #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct CellFlags: u16 {
        const JC1_R1= 1 << 0;
        const JC1_R2= 1 << 1;
        const JC2_R1= 1 << 2;
        const JC2_R2= 1 << 3;
        const NOT_C1 = 1 << 4;
        const NOT_C2= 1 << 5;
        const C1_OUT = 1 << 6;
        const C2_OUT= 1 << 7;
        const R1_OUT= 1 << 8;
        const R2_OUT= 1 << 9;
        const STILL_C1 = 1 << 10;
        const STILL_C2 = 1 << 11;
        const STILL_R1 = 1 << 12;
    }
}

impl Default for CellFlags {
    fn default() -> Self {
        let mut flags = CellFlags::empty();
        flags
            .set_range(10, 3)
            .expect("Unexpected error: invalid range");
        flags
    }
}

impl CellFlags {
    // Sets the various STILL_XY as 1, this is the intended method to create CellFlags
    fn from_bits_checked(bits: u16) -> Self {
        let mut flags = CellFlags::from_bits_truncate(bits);
        flags
            .set_range(10, 3)
            .expect("Unexpected error: invalid range");
        flags
    }

    fn set_range(&mut self, pos: u8, range: u8) -> Result<(), &'static str> {
        if pos > 12 {
            return Err("Position out of range");
        }
        if range == 0 || pos + range > 13 {
            return Err("Invalid range");
        }
        let mask: u16 = ((1 << range) - 1) << pos;
        let new_flags = CellFlags::from_bits_truncate(self.bits() | mask);
        *self = new_flags;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct TargetGroup<const N: usize> {
    target: Selector,
    flags: [CellFlags; N],
}

// Define your groups
impl TargetGroup<5> {
    const C1: TargetGroup<5> = TargetGroup {
        target: Selector::Column1,
        flags: [
            CellFlags::JC1_R1,
            CellFlags::JC1_R2,
            CellFlags::C1_OUT,
            CellFlags::NOT_C1,
            CellFlags::STILL_C1,
        ],
    };

    const C2: TargetGroup<5> = TargetGroup {
        target: Selector::Column2,
        flags: [
            CellFlags::JC2_R1,
            CellFlags::JC2_R2,
            CellFlags::C2_OUT,
            CellFlags::NOT_C2,
            CellFlags::STILL_C2,
        ],
    };
}

impl From<TargetGroup<5>> for TargetGroup<3> {
    fn from(src: TargetGroup<5>) -> Self {
        Self {
            target: src.target,
            flags: [src.flags[0], src.flags[1], src.flags[2]],
        }
    }
}

impl TargetGroup<3> {
    const R1: TargetGroup<3> = TargetGroup {
        target: Selector::Row1,
        flags: [CellFlags::JC1_R1, CellFlags::JC2_R1, CellFlags::R1_OUT],
    };
    const R2: TargetGroup<3> = TargetGroup {
        target: Selector::Row2,
        flags: [CellFlags::JC1_R2, CellFlags::JC2_R2, CellFlags::R2_OUT],
    };
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Fills([u8; 4]);

impl Fills {
    fn new(c1: u8, c2: u8, r1: u8, r2: u8) -> Self {
        Self([c1, c2, r1, r2])
    }

    #[inline]
    fn set(&mut self, target: Selector, val: u8) {
        self.0[target as usize] = val;
    }

    #[inline]
    fn get(&self, target: Selector) -> u8 {
        self.0[target as usize]
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Cell {
    activation_order: ActivationOrder,
    flags: CellFlags,
    fills: Fills,
}

impl Cell {
    const FIXED_BLOCKS: u8 = 9;

    pub fn new(activation_order: &ActivationOrder, flags: &CellFlags, fills: Fills) -> Self {
        let mut flags = flags.clone();
        flags
            .set_range(10, 3)
            .expect("Unexpected error: invalid range");
        Self {
            activation_order: activation_order.clone().clone(),
            flags,
            fills,
        }
    }

    #[inline]
    fn count(&self, input: bool, group: TargetGroup<3>) -> u8 {
        Self::FIXED_BLOCKS
            + self.fills.get(group.target)
            + input as u8
            + (self.flags.contains(group.flags[0]) as u8)
            + (self.flags.contains(group.flags[1]) as u8)
            + (self.flags.contains(group.flags[2]) as u8)
    }

    fn sim_gate(&mut self, column_input: bool, group: TargetGroup<5>) -> bool {
        let mut count: u8 = self.count(column_input, TargetGroup::from(group));

        let out = (self.flags.contains(group.flags[3])
            && !self.flags.contains(CellFlags::STILL_R1))
            || count > 12;

        if !out {
            self.flags.set(group.flags[0], false);
            self.flags.set(group.flags[1], false);
            self.flags.set(group.flags[4], false);
        }
        out
    }

    fn sim_row1(&mut self, row_input: bool) -> bool {
        let mut count: u8 = self.count(row_input, TargetGroup::R1)
            + (self.flags.contains(CellFlags::NOT_C1) as u8)
            + (self.flags.contains(CellFlags::NOT_C2) as u8);

        let out = count > 12
            || (self.flags.contains(CellFlags::NOT_C1)
                && !self.flags.contains(CellFlags::STILL_C1))
            || (self.flags.contains(CellFlags::NOT_C2)
                && !self.flags.contains(CellFlags::STILL_C2));

        if !out {
            self.flags.set(CellFlags::JC1_R1, false);
            self.flags.set(CellFlags::JC2_R1, false);
            self.flags.set(CellFlags::STILL_R1, false);
        }
        out
    }

    fn sim_row2(&mut self, row_input: bool) -> bool {
        let mut count: u8 = self.count(row_input, TargetGroup::R2);

        let out = count > 12;
        if !out {
            self.flags.set(CellFlags::JC1_R2, false);
            self.flags.set(CellFlags::JC2_R2, false);
        }
        out
    }

    pub fn eval_cell(&self, input: CellIO) -> CellIO {
        let mut rtm_cell = self.clone();
        let mut output = CellIO {
            column_1: true,
            column_2: true,
            row_1: true,
            row_2: true,
        };

        for p in rtm_cell.activation_order.0.clone().iter() {
            match p {
                Selector::Column1 => {
                    output.column_1 = rtm_cell.sim_gate(input.column_1, TargetGroup::C1);
                }
                Selector::Column2 => {
                    output.column_2 = rtm_cell.sim_gate(input.column_2, TargetGroup::C2);
                }
                Selector::Row1 => {
                    output.row_1 = rtm_cell.sim_row1(input.row_1);
                }
                Selector::Row2 => {
                    output.row_2 = rtm_cell.sim_row2(input.row_2);
                }
            }
        }
        output
    }
}

#[cfg(test)]
mod cell_tests {
    use crate::fpga::cell::{ActivationOrder, Cell, CellFlags, Fills};

    impl CellFlags {
        const FIXED_BLOCKS: u8 = 9;

        pub(crate) fn new_with_output(
            jc1_r1: bool,
            jc1_r2: bool,
            jc2_r1: bool,
            jc2_r2: bool,
            not_c1: bool,
            not_c2: bool,
        ) -> Self {
            let mut flags = CellFlags::default();
            flags
                .set_range(6, 4)
                .expect("Unexpected error: invalid range");
            flags.set(CellFlags::JC1_R1, jc1_r1);
            flags.set(CellFlags::JC1_R2, jc1_r2);
            flags.set(CellFlags::JC2_R1, jc2_r1);
            flags.set(CellFlags::JC2_R2, jc2_r2);
            flags.set(CellFlags::NOT_C1, not_c1);
            flags.set(CellFlags::NOT_C2, not_c2);
            flags
        }
    }

    use super::*;
    #[test]
    fn activation_order_uniqueness() {
        assert_eq!(
            ActivationOrder::new([
                Selector::Column1,
                Selector::Column2,
                Selector::Row1,
                Selector::Row2
            ]),
            Ok(ActivationOrder([
                Selector::Column1,
                Selector::Column2,
                Selector::Row1,
                Selector::Row2
            ]))
        );

        assert_ne!(
            ActivationOrder::new([
                Selector::Column1,
                Selector::Column2,
                Selector::Row1,
                Selector::Row2
            ]),
            Ok(ActivationOrder([
                Selector::Column1,
                Selector::Row1,
                Selector::Column2,
                Selector::Row2
            ]))
        );

        assert_eq!(
            ActivationOrder::new([
                Selector::Column1,
                Selector::Column2,
                Selector::Row1,
                Selector::Row1
            ]),
            Err("Duplicate enum variants not allowed")
        );
    }

    #[test]
    fn column_evaluation_1() {
        let order = ActivationOrder::new([
            Selector::Column1,
            Selector::Column2,
            Selector::Row1,
            Selector::Row2,
        ])
        .unwrap();

        let flags = CellFlags::new_with_output(false, false, false, false, false, false);

        let fills = Fills::new(0, 0, 0, 0);

        let cell = Cell::new(&order, &flags, fills);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );
    }

    #[test]
    fn column_evaluation_2() {
        let order = ActivationOrder::new([
            Selector::Column1,
            Selector::Column2,
            Selector::Row1,
            Selector::Row2,
        ])
        .unwrap();

        let flags = CellFlags::new_with_output(false, false, false, false, false, false);

        let fills = Fills::new(2, 2, 0, 0);

        let cell = Cell::new(&order, &flags, fills);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );

        let input = CellIO {
            column_1: true,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );
    }

    // Self=1 gate
    #[test]
    fn column_evaluation_3() {
        let order = ActivationOrder::new([
            Selector::Row1,
            Selector::Column1,
            Selector::Column2,
            Selector::Row2,
        ])
        .unwrap();

        let flags = CellFlags::new_with_output(true, true, false, false, false, false);

        println!("{:}", flags.contains(CellFlags::JC1_R2));

        let fills = Fills::new(2, 0, 0, 0);

        let cell = Cell::new(&order, &flags, fills);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );

        let input = CellIO {
            column_1: true,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );

        let fills = Fills::new(2, 0, 5, 0);

        let cell = Cell::new(&order, &flags, fills);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: true,
                row_2: false,
            }
        );
    }

    #[test]
    fn column_evaluation_4() {
        let order = ActivationOrder::new([
            Selector::Column1,
            Selector::Column2,
            Selector::Row1,
            Selector::Row2,
        ])
        .unwrap();

        let flags = CellFlags::new_with_output(true, false, false, false, false, false);

        let fills = Fills::new(1, 0, 2, 0);

        let cell = Cell::new(&order, &flags, fills);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        cell.eval_cell(input);

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );

        let input = CellIO {
            column_1: true,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: true,
                row_2: false,
            }
        );
    }

    #[test]
    fn column_evaluation_5() {
        let order = ActivationOrder::new([
            Selector::Column1,
            Selector::Column2,
            Selector::Row1,
            Selector::Row2,
        ])
        .unwrap();

        let flags = CellFlags::new_with_output(true, true, false, false, false, false);

        let fills = Fills::new(0, 0, 2, 2);

        let cell = Cell::new(&order, &flags, fills);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        cell.eval_cell(input);

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );

        let input = CellIO {
            column_1: true,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: true,
                row_2: true,
            }
        );
    }

    #[test]
    fn column_evaluation_6() {
        let order = ActivationOrder::new([
            Selector::Row1,
            Selector::Column1,
            Selector::Row2,
            Selector::Column2,
        ])
        .unwrap();

        let flags = CellFlags::new_with_output(true, true, false, false, false, false);

        let fills = Fills::new(0, 0, 1, 2);

        let cell = Cell::new(&order, &flags, fills);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );

        let input = CellIO {
            column_1: true,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: true,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: true,
                row_2: false,
            }
        );

        let input = CellIO {
            column_1: true,
            column_2: false,
            row_1: true,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: true,
                row_2: true,
            }
        );
    }

    #[test]
    fn column_evaluation_7() {
        let order = ActivationOrder::new([
            Selector::Row1,
            Selector::Column1,
            Selector::Row2,
            Selector::Column2,
        ])
        .unwrap();

        let flags = CellFlags::new_with_output(true, true, false, false, false, false);

        let fills = Fills::new(1, 0, 1, 2);

        let cell = Cell::new(&order, &flags, fills);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );

        let input = CellIO {
            column_1: true,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: false,
                row_2: true,
            }
        );

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: true,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: true,
                row_2: true,
            }
        );

        let input = CellIO {
            column_1: true,
            column_2: false,
            row_1: true,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: true,
                row_2: true,
            }
        );
    }

    #[test]
    fn not_column_evaluation_1() {
        let order = ActivationOrder::new([
            Selector::Row1,
            Selector::Column1,
            Selector::Row2,
            Selector::Column2,
        ])
        .unwrap();

        let flags = CellFlags::new_with_output(false, false, false, false, true, false);

        let fills = Fills::new(0, 0, 1, 0);

        let cell = Cell::new(&order, &flags, fills);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: true,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: true,
                row_2: false,
            }
        );
    }

    #[test]
    fn not_column_evaluation_2() {
        let order = ActivationOrder::new([
            Selector::Row1,
            Selector::Row2,
            Selector::Column1,
            Selector::Column2,
        ])
        .unwrap();

        let flags = CellFlags::new_with_output(false, true, false, false, true, false);

        let fills = Fills::new(1, 0, 1, 1);

        let cell = Cell::new(&order, &flags, fills);

        let mut input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );

        input = CellIO {
            column_1: false,
            column_2: false,
            row_1: true,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: true,
                row_2: false,
            }
        );

        input = CellIO {
            column_1: true,
            column_2: false,
            row_1: true,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: true,
                row_2: false,
            }
        );

        input = CellIO {
            column_1: true,
            column_2: false,
            row_1: true,
            row_2: false,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: true,
                row_2: false,
            }
        );

        input = CellIO {
            column_1: false,
            column_2: false,
            row_1: true,
            row_2: true,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: true,
                row_2: true,
            }
        );

        input = CellIO {
            column_1: true,
            column_2: false,
            row_1: true,
            row_2: true,
        };

        assert_eq!(
            cell.eval_cell(input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: true,
                row_2: true,
            }
        );
    }
}
