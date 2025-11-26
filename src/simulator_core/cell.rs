//! This module represent the single FPGA cell.
//! It contains all the methods and functions
//! needed to configure, interact and simulate
//! each individual FPGA cell.
//!
//! I made large usage of the [bitflags] create
//! to optimize the cache hits and misses so that
//! the simulation can run as fast as possible.

use crate::impl_set_range;
use bitflags::{Flags, bitflags};
use serde::de::EnumAccess;
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

/// This struct is used to describe in which order the cell
/// columns and rows activate. This order is crucial in
/// defining what will be the logic function characteristic
/// of the [Cell].
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ActivationOrder([Selector; 4]);

impl IntoIterator for ActivationOrder {
    type Item = Selector;
    type IntoIter = core::array::IntoIter<Selector, 4>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Default for ActivationOrder {
    /// The default [ActivationOrder] of a cell.
    /// This is no particularly special order,
    /// any other order is fine as default.
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
    /// This function creates a new [ActivationOrder]
    /// instance.
    ///
    /// ## Arguments
    ///
    /// - `order`: A 4 long array of [Selector].
    ///
    /// ## Returns
    ///
    /// - [Ok(ActivationOrder)] if `order` contains no duplicates.
    /// - [Err()] if `order` contains any duplicate.
    ///
    /// ## Example
    ///
    /// ```
    /// use simulator_core::cell::{ActivationOrder, Selector};
    /// // Successfully creates and returns a new activation order: Column1 -> Row2 -> Row1 -> Column2.
    /// assert!(ActivationOrder::new([Selector::Column1, Selector::Row2, Selector::Row1, Selector::Column2]).is_ok());
    /// // Returns an Err() because the input array contains a duplicate value.
    /// assert!(ActivationOrder::new([Selector::Column1, Selector::Column1, Selector::Row1, Selector::Row2]).is_err());
    /// ```
    pub fn new(order: [Selector; 4]) -> Result<Self, &'static str> {
        let set: HashSet<_> = order.iter().collect();
        if set.len() != 4 {
            return Err("Duplicate enum variants not allowed");
        }
        Ok(ActivationOrder(order))
    }
}

/// This struct represents the amount of filler
/// blocks on each [Cell] line.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Fills([u8; 4]);

impl Fills {
    #[inline]
    fn new(c1: u8, c2: u8, r1: u8, r2: u8) -> Self {
        Self([c1, c2, r1, r2])
    }

    /// Sets the amount of filler blocks of the given line.
    #[inline]
    fn set(&mut self, target: u8, val: u8) {
        self.0[target as usize] = val;
    }

    /// Gets the amount of filler blocks of the given line.
    #[inline]
    fn get(&self, target: u8) -> u8 {
        self.0[target as usize]
    }
}

bitflags! {
    /// This represents the input/output blocks that connect
    /// one [Cell] to the previous/next one.
    /// It's mainly used for simulation purposes.
    #[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
    pub struct CellIO: u8 {
        const COLUMN_1 = 1 << 0;
        const COLUMN_2 = 1 << 1;
        const ROW_1 = 1 << 2;
        const ROW_2 = 1 << 3;
    }
}

impl CellIO {
    #[inline]
    pub fn new(c1: bool, c2: bool, r1: bool, r2: bool) -> Self {
        let mut var = CellIO::empty();
        var.set(CellIO::COLUMN_1, c1);
        var.set(CellIO::COLUMN_2, c2);
        var.set(CellIO::ROW_1, r1);
        var.set(CellIO::ROW_2, r2);
        var
    }

    #[inline]
    pub fn contains_as_u8(&self, flag: CellIO) -> u8 {
        (*self & flag).bits() >> flag.bits().trailing_zeros()
    }
}

bitflags! {
    /// This represents the inner configuration of the [Cell]
    /// blocks and of its outputs.
    ///
    /// ## Safety
    ///
    /// The flags(
    /// [`STILL_C1`](CellFlags::STILL_C1),
    /// [`STILL_C2`](CellFlags::STILL_C2),
    /// [`STILL_R1`](CellFlags::STILL_R1))
    /// must be set to 1 on creation otherwise you will incur in
    /// simulation errors.
    ///
    /// ## Note
    ///
    /// The previous mentioned flags are only
    /// used for simulation purposes (
    /// [`STILL_C1`](CellFlags::STILL_C1),
    /// [`STILL_C2`](CellFlags::STILL_C2),
    /// [`STILL_R1`](CellFlags::STILL_R1)
    /// ).
    ///
    /// We don't have a param `STILL_R2` since these parameters
    /// are only used for the NOT function calculation and, since
    /// Row 2 has no such function due to design limitations, it
    /// isn't needed.
    ///
    /// The reasons why I opted to put these flags in here instead
    /// of another dedicated structure is that we already needed
    /// 9 bits and thus had to use a [u16]. Creating a dedicated
    /// [u8] bitflag would've just increased the cache misses without
    /// any other benefit, since we would be using 24 bits instead of
    /// 16.
    #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct CellFlags: u16 {
        // Junction between Col 1 and Row 1.
        const JC1_R1 = 1 << 0;
        // Junction between Col 1 and Row 2.
        const JC1_R2 = 1 << 1;
        // Junction between Col 2 and Row 1.
        const JC2_R1 = 1 << 2;
        // Junction between Col 2 and Row 2.
        const JC2_R2 = 1 << 3;

        // Not function on Col 1.
        const NOT_C1 = 1 << 4;
        // Not function on Col 2.
        const NOT_C2 = 1 << 5;

        // Output of Col 1.
        const C1_OUT = 1 << 6;
        // Output of Col 2.
        const C2_OUT = 1 << 7;
        // Output of Row 1.
        const R1_OUT = 1 << 8;
        // Output of Row 2.
        const R2_OUT = 1 << 9;

        // These parameters are only relevant for
        // the NOT function calculation.
        //
        // Remember that still=true, move=false.

        // If Col 1 stood still(or didn't move yet, same thing).
        const STILL_C1 = 1 << 10;
        // If Col 2 stood still(or didn't move yet, same thing).
        const STILL_C2 = 1 << 11;
        // If Row 1 stood still(or didn't move yet, same thing).
        const STILL_R1 = 1 << 12;
    }
}

// This just calls the impl_set_range!() macro that
// I created to not write the same implementation
// again and again for each bitflag.
impl_set_range!(CellIO, CellFlags);

impl Default for CellFlags {
    /// This returns an empty CellFlags instance
    /// with all the flags set to 0 except the
    /// 3 simulation-only related flags(see
    /// [CellFlags] docs for more information).
    #[inline]
    fn default() -> Self {
        let mut flags = CellFlags::empty();
        flags.set_range(10, 3);
        flags
    }
}

impl CellFlags {
    /// This converts the given `bits` to a [CellFlags]
    /// and sets the various STILL_XY flags to 1 as required.
    ///
    /// ## Note
    ///
    /// It will unset any unknown bits.
    #[inline]
    fn from_bits_checked(bits: u16) -> Self {
        let mut flags = CellFlags::from_bits_truncate(bits);
        flags.set_range(10, 3);
        flags
    }
}

/// This is mostly a struct used to generalise some
/// simulation-related functions.
/// There are 4 target groups defined, one for each line:
/// - [C1](TargetGroup::C1): represents Col 1.
/// - [C2](TargetGroup::C2): represents Col 2.
/// - [R1](TargetGroup::R1): represents Row 1.
/// - [R2](TargetGroup::R2): represents Row 2.
///
/// ## Arguments
///
/// - `target`: what [Fills] index represents that given line.
/// - `flags`: this is an array of [CellFlags] const(not of
///    instances), it represents the set of [CellFlags] flags
///    relevant to that line.
///
#[derive(Debug, Clone, Copy)]
struct TargetGroup<const N: usize> {
    target: u8,
    cell_io: CellIO,
    flags: [CellFlags; N],
}

impl TargetGroup<5> {
    const C1: TargetGroup<5> = TargetGroup {
        target: 0,
        cell_io: CellIO::COLUMN_1,
        flags: [
            CellFlags::JC1_R1,
            CellFlags::JC1_R2,
            CellFlags::C1_OUT,
            CellFlags::NOT_C1,
            CellFlags::STILL_C1,
        ],
    };

    const C2: TargetGroup<5> = TargetGroup {
        target: 1,
        cell_io: CellIO::COLUMN_2,
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
    #[inline]
    fn from(src: TargetGroup<5>) -> Self {
        Self {
            target: src.target,
            cell_io: src.cell_io,
            flags: [src.flags[0], src.flags[1], src.flags[2]],
        }
    }
}

impl TargetGroup<3> {
    const R1: TargetGroup<3> = TargetGroup {
        target: 2,
        cell_io: CellIO::ROW_1,
        flags: [CellFlags::JC1_R1, CellFlags::JC2_R1, CellFlags::R1_OUT],
    };
    const R2: TargetGroup<3> = TargetGroup {
        target: 3,
        cell_io: CellIO::ROW_2,
        flags: [CellFlags::JC1_R2, CellFlags::JC2_R2, CellFlags::R2_OUT],
    };
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Cell {
    pub activation_order: ActivationOrder,
    pub flags: CellFlags,
    pub fills: Fills,
}

type LineEvalFn = fn(&mut Cell, &mut CellIO);

impl Cell {
    /// The fixed amount of blocks that each line is made of.
    const FIXED_BLOCKS: u8 = 9;
    const EVAL_TABLE: [LineEvalFn; 4] = [
        Self::sim_col1,
        Self::sim_col2,
        Self::sim_row1,
        Self::sim_row2,
    ];

    #[inline]
    pub fn new(activation_order: &ActivationOrder, flags: &CellFlags, fills: Fills) -> Self {
        let mut flags = flags.clone();
        flags.set_range(10, 3);
        Self {
            activation_order: activation_order.clone().clone(),
            flags,
            fills,
        }
    }

    /// Calculates the amount of blocks on the given `group`.
    #[inline]
    fn count(&self, input: CellIO, group: TargetGroup<3>) -> u8 {
        Self::FIXED_BLOCKS
            + self.fills.get(group.target)
            + input.contains_as_u8(group.cell_io)
            + (self.flags.contains(group.flags[0]) as u8)
            + (self.flags.contains(group.flags[1]) as u8)
            + (self.flags.contains(group.flags[2]) as u8)
    }

    /// Simulates the specified column with the specified inputs.
    ///
    /// ## Arguments
    ///
    /// - `column_input`:
    #[inline]
    fn sim_column(&mut self, mut input: &mut CellIO, group: TargetGroup<5>) {
        let mut count: u8 = self.count(*input, TargetGroup::from(group));

        let out = (self.flags.contains(group.flags[3])
            && !self.flags.contains(CellFlags::STILL_R1))
            || count > 12;

        if !out {
            self.flags.set(group.flags[0], false);
            self.flags.set(group.flags[1], false);
            self.flags.set(group.flags[4], false);
        }

        input.set(group.cell_io, out);
    }

    #[inline(always)]
    fn sim_col1(&mut self, input: &mut CellIO) {
        self.sim_column(input, TargetGroup::C1);
    }

    #[inline(always)]
    fn sim_col2(&mut self, input: &mut CellIO) {
        self.sim_column(input, TargetGroup::C2);
    }

    #[inline]
    fn sim_row1(&mut self, mut input: &mut CellIO) {
        let mut count: u8 = self.count(*input, TargetGroup::R1)
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
        input.set(CellIO::ROW_1, out);
    }

    #[inline]
    fn sim_row2(&mut self, mut input: &mut CellIO) {
        let mut count: u8 = self.count(*input, TargetGroup::R2);

        let out = count > 12;
        if !out {
            self.flags.set(CellFlags::JC1_R2, false);
            self.flags.set(CellFlags::JC2_R2, false);
        }
        input.set(CellIO::ROW_2, out);
    }

    #[inline]
    pub fn eval_cell(&self, mut input: CellIO) -> CellIO {
        let mut rtm_cell = self.clone();

        for selector in rtm_cell.activation_order.0.clone().iter() {
            Self::EVAL_TABLE[*selector as usize](&mut rtm_cell, &mut input);
        }

        input
    }

    #[inline]
    pub fn print_truth_table(&self) {
        let header = [
            "C1", "C2", "R1", "R2", "C1 Out", "C2 Out", "R1 Out", "R2 Out",
        ];

        println!("+-----+-----+-----+-----+---------+---------+---------+---------+");
        println!(
            "| {:<3} | {:<3} | {:<3} | {:<3} | {:<7} | {:<7} | {:<7} | {:<7} |",
            header[0], header[1], header[2], header[3], header[4], header[5], header[6], header[7]
        );
        println!("+-----+-----+-----+-----+---------+---------+---------+---------+");

        for i in (0..16).rev() {
            let input = CellIO::from_bits_truncate(i as u8);
            let eval = self.eval_cell(input);

            println!(
                "| {:<3} | {:<3} | {:<3} | {:<3} | {:<7} | {:<7} | {:<7} | {:<7} |",
                input.contains_as_u8(CellIO::COLUMN_1),
                input.contains_as_u8(CellIO::COLUMN_2),
                input.contains_as_u8(CellIO::ROW_1),
                input.contains_as_u8(CellIO::ROW_2),
                eval.contains_as_u8(CellIO::COLUMN_1),
                eval.contains_as_u8(CellIO::COLUMN_2),
                eval.contains_as_u8(CellIO::ROW_1),
                eval.contains_as_u8(CellIO::ROW_2),
            );
        }

        println!("+-----+-----+-----+-----+---------+---------+---------+---------+");
    }
}

#[cfg(test)]
mod cell_tests {
    use super::{ActivationOrder, Cell, CellFlags, Fills};

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
            flags.set_range(6, 4);
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
    fn cell_io_contains_as_bits() {
        let cell = CellIO::new(false, true, false, false);
        assert_eq!(cell.contains_as_u8(CellIO::COLUMN_1), 0);
        assert_eq!(cell.contains_as_u8(CellIO::COLUMN_2), 1);
        assert_eq!(cell.contains_as_u8(CellIO::ROW_1), 0);
        assert_eq!(cell.contains_as_u8(CellIO::ROW_2), 0);

        let cell = CellIO::new(true, true, false, true);
        assert_eq!(cell.contains_as_u8(CellIO::COLUMN_1), 1);
        assert_eq!(cell.contains_as_u8(CellIO::COLUMN_2), 1);
        assert_eq!(cell.contains_as_u8(CellIO::ROW_1), 0);
        assert_eq!(cell.contains_as_u8(CellIO::ROW_2), 1);
    }

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

        let input = CellIO::new(false, false, false, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, false, false)
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

        let input = CellIO::new(false, false, false, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, false, false)
        );

        let input = CellIO::new(true, false, false, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(true, false, false, false)
        );

        cell.print_truth_table();
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

        let fills = Fills::new(2, 0, 0, 0);

        let cell = Cell::new(&order, &flags, fills);

        let input = CellIO::new(false, false, false, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(true, false, false, false)
        );

        let input = CellIO::new(true, false, false, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(true, false, false, false)
        );

        let fills = Fills::new(2, 0, 5, 0);

        let cell = Cell::new(&order, &flags, fills);

        let input = CellIO::new(false, false, false, false);

        assert_eq!(cell.eval_cell(input), CellIO::new(true, false, true, false));
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

        let input = CellIO::new(false, false, false, false);

        cell.eval_cell(input);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, false, false)
        );

        let input = CellIO::new(true, false, false, false);

        assert_eq!(cell.eval_cell(input), CellIO::new(true, false, true, false));
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

        let input = CellIO::new(false, false, false, false);

        cell.eval_cell(input);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, false, false)
        );

        let input = CellIO::new(true, false, false, false);

        assert_eq!(cell.eval_cell(input), CellIO::new(true, false, true, true));
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

        let input = CellIO::new(false, false, false, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, false, false)
        );

        let input = CellIO::new(true, false, false, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, false, false)
        );

        let input = CellIO::new(false, false, true, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, true, false)
        );

        let input = CellIO::new(true, false, true, false);

        assert_eq!(cell.eval_cell(input), CellIO::new(true, false, true, true));
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

        let input = CellIO::new(false, false, false, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, false, false)
        );

        let input = CellIO::new(true, false, false, false);

        assert_eq!(cell.eval_cell(input), CellIO::new(true, false, false, true));

        let input = CellIO::new(false, false, true, false);

        assert_eq!(cell.eval_cell(input), CellIO::new(true, false, true, true));

        let input = CellIO::new(true, false, true, false);

        assert_eq!(cell.eval_cell(input), CellIO::new(true, false, true, true));
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

        let input = CellIO::new(false, false, false, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(true, false, false, false)
        );

        let input = CellIO::new(false, false, true, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, true, false)
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

        let mut input = CellIO::new(false, false, false, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(true, false, false, false)
        );

        input = CellIO::new(false, false, true, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, true, false)
        );

        input = CellIO::new(true, false, true, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, true, false)
        );

        input = CellIO::new(true, false, true, false);

        assert_eq!(
            cell.eval_cell(input),
            CellIO::new(false, false, true, false)
        );

        input = CellIO::new(false, false, true, true);

        assert_eq!(cell.eval_cell(input), CellIO::new(false, false, true, true));

        input = CellIO::new(true, false, true, true);

        assert_eq!(cell.eval_cell(input), CellIO::new(true, false, true, true));
    }
}