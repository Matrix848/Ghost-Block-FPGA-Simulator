use std::collections::HashSet;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub(crate) enum Priorities {
    COLUMN1,
    COLUMN2,
    ROW1,
    ROW2,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub(crate) struct ActivationOrder([Priorities; 4]);

impl ActivationOrder {
    pub(crate) fn new(order: [Priorities; 4]) -> Result<Self, &'static str> {
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

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    activation_order: ActivationOrder,
    // Column 1 configuration
    fill_c1: u8,
    not_1: bool,
    // Column 1, row 1
    j1_1: bool,
    // Column 1, row 2
    j1_2: bool,

    // Column 2 configuration
    fill_c2: u8,
    not_2: bool,
    // Column 2, row 1
    j2_1: bool,
    // Column 2, row 2
    j2_2: bool,

    // Row 1 configuration
    fill_l1: u8,
    // Row 2 configuration
    fill_l2: u8,
}

impl Cell {
    pub(crate) fn new(
        activation_order: ActivationOrder,
        fill_c1: u8,
        not_1: bool,
        j1_1: bool,
        j1_2: bool,
        fill_c2: u8,
        not_2: bool,
        j2_1: bool,
        j2_2: bool,
        fill_l1: u8,
        fill_l2: u8,
    ) -> Self {
        Self {
            activation_order,
            fill_c1,
            not_1,
            j1_1,
            j1_2,
            fill_c2,
            not_2,
            j2_1,
            j2_2,
            fill_l1,
            fill_l2,
        }
    }

    pub fn eval(&self, input: CellIO) -> CellIO {
        RuntimeCell::eval_cell(self, input)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeCell {
    still_column_1: bool,
    still_column_2: bool,
    still_row_1: bool,
    cell: Cell,
}

impl RuntimeCell {
    // Note: x_1 is the block, still_row_1 tells us if the row still(and thus the ghost NOT block still)
    fn eval_gate(&self, fill: u8, column_input: bool, jx_1: bool, jx_2: bool, not: bool) -> bool {
        let mut count: u8 = 10 + fill;

        if column_input {
            count += 1
        }
        if jx_1 {
            count += 1
        }
        if jx_2 {
            count += 1
        }

        (not && !self.still_row_1) || count > 12
    }

    fn eval_row(&self, fill: u8, row_input: bool, j1_y: bool, j2_y: bool, not_row: bool) -> bool {
        let mut count: u8 = 10 + fill;

        if row_input {
            count += 1
        }
        if j1_y {
            count += 1
        }
        if j2_y {
            count += 1
        }

        if not_row {
            if self.cell.not_1 {
                count += 1
            }
            if self.cell.not_2 {
                count += 1
            }

            return count > 12
                || (self.cell.not_1 && !self.still_column_1)
                || (self.cell.not_2 && !self.still_column_2);
        }
        count > 12
    }
    fn eval_cell(cell: &Cell, input: CellIO) -> CellIO {
        let mut rtm_cell = RuntimeCell {
            still_column_1: true,
            still_column_2: true,
            still_row_1: true,
            cell: cell.clone(),
        };
        let mut output = CellIO {
            column_1: true,
            column_2: true,
            row_1: true,
            row_2: true,
        };

        for p in rtm_cell.cell.activation_order.0.iter() {
            match p {
                Priorities::COLUMN1 => {
                    output.column_1 = rtm_cell.eval_gate(
                        rtm_cell.cell.fill_c1,
                        input.column_1,
                        rtm_cell.cell.j1_1,
                        rtm_cell.cell.j1_2,
                        rtm_cell.cell.not_1,
                    );
                    if !output.column_1 {
                        (
                            rtm_cell.cell.j1_1,
                            rtm_cell.cell.j1_2,
                            rtm_cell.still_column_1,
                        ) = (false, false, false);
                    }
                }
                Priorities::COLUMN2 => {
                    output.column_2 = rtm_cell.eval_gate(
                        rtm_cell.cell.fill_c2,
                        input.column_2,
                        rtm_cell.cell.j2_1,
                        rtm_cell.cell.j2_2,
                        rtm_cell.cell.not_2,
                    );
                    if !output.column_2 {
                        (
                            rtm_cell.cell.j2_1,
                            rtm_cell.cell.j2_2,
                            rtm_cell.still_column_2,
                        ) = (false, false, false);
                    }
                }
                Priorities::ROW1 => {
                    output.row_1 = rtm_cell.eval_row(
                        rtm_cell.cell.fill_l1,
                        input.row_1,
                        rtm_cell.cell.j1_1,
                        rtm_cell.cell.j2_1,
                        true,
                    );
                    if !output.row_1 {
                        (rtm_cell.cell.j1_1, rtm_cell.cell.j2_1, rtm_cell.still_row_1) =
                            (false, false, false);
                    }
                }
                Priorities::ROW2 => {
                    output.row_2 = rtm_cell.eval_row(
                        rtm_cell.cell.fill_l2,
                        input.row_2,
                        rtm_cell.cell.j1_2,
                        rtm_cell.cell.j2_2,
                        false,
                    );
                    if !output.row_2 {
                        (rtm_cell.cell.j1_2, rtm_cell.cell.j2_2) = (false, false);
                    }
                }
            }
        }
        output
    }
}

#[cfg(test)]
mod cell_tests {

    use super::*;
    #[test]
    fn activation_order_uniqueness() {
        assert_eq!(
            ActivationOrder::new([
                Priorities::COLUMN1,
                Priorities::COLUMN2,
                Priorities::ROW1,
                Priorities::ROW2
            ]),
            Ok(ActivationOrder([
                Priorities::COLUMN1,
                Priorities::COLUMN2,
                Priorities::ROW1,
                Priorities::ROW2
            ]))
        );

        assert_ne!(
            ActivationOrder::new([
                Priorities::COLUMN1,
                Priorities::COLUMN2,
                Priorities::ROW1,
                Priorities::ROW2
            ]),
            Ok(ActivationOrder([
                Priorities::COLUMN1,
                Priorities::ROW1,
                Priorities::COLUMN2,
                Priorities::ROW2
            ]))
        );

        assert_eq!(
            ActivationOrder::new([
                Priorities::COLUMN1,
                Priorities::COLUMN2,
                Priorities::ROW1,
                Priorities::ROW1
            ]),
            Err("Duplicate enum variants not allowed")
        );
    }

    #[test]
    fn column_evaluation_1() {
        let order_1 = ActivationOrder::new([
            Priorities::COLUMN1,
            Priorities::COLUMN2,
            Priorities::ROW1,
            Priorities::ROW2,
        ])
        .unwrap();

        let cell = Cell::new(
            order_1, 0, false, false, false, 0, false, false, false, 0, 0,
        );

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            RuntimeCell::eval_cell(&cell, input),
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
        let order_1 = ActivationOrder::new([
            Priorities::COLUMN1,
            Priorities::COLUMN2,
            Priorities::ROW1,
            Priorities::ROW2,
        ])
        .unwrap();

        let cell = Cell::new(
            order_1, 2, false, false, false, 2, false, false, false, 0, 0,
        );

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
        let order_1 = ActivationOrder::new([
            Priorities::ROW1,
            Priorities::COLUMN1,
            Priorities::COLUMN2,
            Priorities::ROW2,
        ])
        .unwrap();

        let cell = Cell::new(order_1, 2, true, true, false, 0, false, false, false, 0, 0);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );

        let cell = Cell::new(order_1, 2, true, true, false, 0, false, false, false, 5, 0);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            RuntimeCell::eval_cell(&cell, input),
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
        let order_1 = ActivationOrder::new([
            Priorities::COLUMN1,
            Priorities::COLUMN2,
            Priorities::ROW1,
            Priorities::ROW2,
        ])
        .unwrap();

        let cell = Cell::new(order_1, 1, false, true, false, 0, false, false, false, 2, 0);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        RuntimeCell::eval_cell(&cell, input);

        assert_eq!(
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
        let order_1 = ActivationOrder::new([
            Priorities::COLUMN1,
            Priorities::COLUMN2,
            Priorities::ROW1,
            Priorities::ROW2,
        ])
        .unwrap();

        let cell = Cell::new(order_1, 0, false, true, true, 0, false, false, false, 2, 2);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        RuntimeCell::eval_cell(&cell, input);

        assert_eq!(
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
        let order_1 = ActivationOrder::new([
            Priorities::ROW1,
            Priorities::COLUMN1,
            Priorities::ROW2,
            Priorities::COLUMN2,
        ])
        .unwrap();

        let cell = Cell::new(order_1, 0, false, true, true, 0, false, false, false, 1, 2);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
        let order_1 = ActivationOrder::new([
            Priorities::ROW1,
            Priorities::COLUMN1,
            Priorities::ROW2,
            Priorities::COLUMN2,
        ])
        .unwrap();

        let cell = Cell::new(order_1, 1, false, true, true, 0, false, false, false, 1, 2);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
        let order_1 = ActivationOrder::new([
            Priorities::ROW1,
            Priorities::COLUMN1,
            Priorities::ROW2,
            Priorities::COLUMN2,
        ])
        .unwrap();

        let cell = Cell::new(order_1, 0, true, false, false, 0, false, false, false, 1, 0);

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
        let order_1 = ActivationOrder::new([
            Priorities::ROW1,
            Priorities::ROW2,
            Priorities::COLUMN1,
            Priorities::COLUMN2,
        ])
        .unwrap();

        let cell = Cell::new(order_1, 1, true, false, true, 0, false, false, false, 1, 1);

        let mut input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
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
            RuntimeCell::eval_cell(&cell, input),
            CellIO {
                column_1: true,
                column_2: false,
                row_1: true,
                row_2: true,
            }
        );
    }

    #[test]
    fn column_evaluation_8() {
        let order_1 = ActivationOrder::new([
            Priorities::COLUMN1,
            Priorities::COLUMN2,
            Priorities::ROW1,
            Priorities::ROW2,
        ])
        .unwrap();

        let cell = Cell::new(
            order_1, 0, false, false, false, 0, false, false, false, 0, 0,
        );

        let input = CellIO {
            column_1: false,
            column_2: false,
            row_1: false,
            row_2: false,
        };

        assert_eq!(
            RuntimeCell::eval_cell(&cell, input),
            CellIO {
                column_1: false,
                column_2: false,
                row_1: false,
                row_2: false,
            }
        );
    }
}
