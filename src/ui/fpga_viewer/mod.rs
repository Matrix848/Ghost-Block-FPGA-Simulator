use crate::io::File;
use crate::ui::Message;
use iced::widget::{Column, Container, Row, Space, container, text};
use iced::{Background, Color, Length, Renderer, Theme};
use iced_aw::{Grid, GridRow};
use simulator_core::cell::{ActivationOrder, CellFlags};

#[derive(Debug, Default)]
pub(crate) struct FpgaViewer {
    pub file: File,
}

impl FpgaViewer {
    const NOT_COLOR: Color = Color::from_rgb(0.45, 0.0, 0.0);
    const NORMAL_COLOR: Color = Color::from_rgb(0.29, 0.29, 0.32);
    const JUNCTION_COLOR: Color = Color::from_rgb(0.05, 0.9, 0.8);
    const OUT_COLOR: Color = Color::from_rgb(0.82, 0.05, 0.88);

    #[inline]
    pub(crate) fn view(&self) -> Grid<'_, Message, Theme, Renderer> {
        let mut grid = Grid::new();

        if self.file.fpga.height() == 0 || self.file.fpga.width() == 1 {
            println!("error");
            return grid;
        }

        println!(
            "H: {}, W: {}",
            self.file.fpga.height(),
            self.file.fpga.width()
        );

        for row in (0..self.file.fpga.height() - 1).rev() {
            let mut grid_row: GridRow<'_, Message, Theme, Renderer> = GridRow::new();
            for col in 0..self.file.fpga.width() - 1 {
                grid_row = grid_row.push(self.cell(row, col));
            }
            grid = grid.push(grid_row)
        }

        grid
    }

    #[inline]
    pub(crate) fn cell(&self, row: usize, col: usize) -> Column<'_, Message, Theme, Renderer> {
        let cell_data = self
            .file
            .get_cell(row, col)
            .expect("Internal Error: cell not found");

        let flags = &cell_data.flags;

        let mut column = Column::new().spacing(0);

        let empty = || pixel(Color::TRANSPARENT);

        let row_1 = || pixel(Self::NORMAL_COLOR);
        let row_2 = || pixel(Self::NORMAL_COLOR);

        let col_1 = || not_pixel(CellFlags::NOT_C1, flags);
        let col_2 = || not_pixel(CellFlags::NOT_C2, flags);

        let jc1_r1 = junction_pixel(CellFlags::JC1_R1, flags);
        let jc1_r2 = junction_pixel(CellFlags::JC1_R2, flags);
        let jc2_r1 = junction_pixel(CellFlags::JC2_R1, flags);
        let jc2_r2 = junction_pixel(CellFlags::JC2_R2, flags);

        let row_1_out = out_pixel(CellFlags::C1_OUT, flags);
        let row_2_out = out_pixel(CellFlags::C1_OUT, flags);
        let col_1_out = out_pixel(CellFlags::C1_OUT, flags);
        let col_2_out = out_pixel(CellFlags::C1_OUT, flags);

        let [col_1_order, col_2_order, row_1_order, row_2_order] =
            order_pixels(&cell_data.activation_order);

        let mut row = Row::new().spacing(0);

        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_1_out);
        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_2_out);
        row = row.push(empty());
        row = row.push(empty());

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_1());
        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_2());
        row = row.push(empty());
        row = row.push(empty());

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        row = row.push(row_2_out);
        row = row.push(row_2());
        row = row.push(jc2_r2);
        row = row.push(row_2());
        row = row.push(row_2());
        row = row.push(jc1_r2);
        row = row.push(row_2());
        row = row.push(row_2_order);

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_1());
        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_2());
        row = row.push(empty());
        row = row.push(empty());

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_1());
        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_2());
        row = row.push(empty());
        row = row.push(empty());

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        row = row.push(row_1_out);
        row = row.push(row_2());
        row = row.push(jc2_r1);
        row = row.push(row_1());
        row = row.push(row_1());
        row = row.push(jc1_r1);
        row = row.push(row_1());
        row = row.push(row_1_order);

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_1());
        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_2());
        row = row.push(empty());
        row = row.push(empty());

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_2_order);
        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_1_order);
        row = row.push(empty());
        row = row.push(empty());

        column = column.push(row);

        column
    }
}

fn order_pixels<'a>(
    activation_order: &ActivationOrder,
) -> [Container<'a, Message, Theme, Renderer>; 4] {
    let mut vec: [Container<Message, Theme, Renderer>; 4] =
        std::array::from_fn(|_| pixel(Color::TRANSPARENT));
    for (i, selector) in activation_order.into_iter().enumerate() {
        vec[selector as usize] = container(text(i))
            .width(Length::Fixed(12.))
            .height(Length::Fixed(12.))
            .style(move |_theme| container::Style {
                background: Some(Background::Color(FpgaViewer::NORMAL_COLOR)),
                ..Default::default()
            });
    }
    vec
}

#[inline]
fn out_pixel(out: CellFlags, cell_flags: &CellFlags) -> Container<'_, Message, Theme, Renderer> {
    let tmp = if cell_flags.contains(out) {
        FpgaViewer::OUT_COLOR
    } else {
        Color::TRANSPARENT
    };
    pixel(tmp)
}

#[inline]
fn not_pixel(not: CellFlags, cell_flags: &CellFlags) -> Container<'_, Message, Theme, Renderer> {
    let tmp = if cell_flags.contains(not) {
        FpgaViewer::NOT_COLOR
    } else {
        FpgaViewer::NORMAL_COLOR
    };
    pixel(tmp)
}

#[inline]
fn junction_pixel(
    junction: CellFlags,
    cell_flags: &CellFlags,
) -> Container<'_, Message, Theme, Renderer> {
    let tmp = if cell_flags.contains(junction) {
        FpgaViewer::JUNCTION_COLOR
    } else {
        FpgaViewer::NORMAL_COLOR
    };
    pixel(tmp)
}

#[inline]
pub fn pixel<'a>(color: Color) -> Container<'a, Message, Theme, Renderer> {
    container(Space::new(Length::Fixed(12.), Length::Fixed(12.))).style(move |_theme| {
        container::Style {
            background: Some(Background::Color(color)),
            ..container::Style::default()
        }
    })
}
