use crate::gui::Message;
use crate::io::File;
use iced::widget::{Column, Container, Row, Space, container, text};
use iced::{Background, Color, Length, Renderer, Theme};
use iced_aw::{Grid, GridRow};
use simulator_core::cell::{ActivationOrder, CellFlags};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub(crate) struct FpgaViewer {
    pub(crate) file_resource: Arc<RwLock<File>>,
    pixel_size: f32,
}
impl FpgaViewer {
    const NOT_COLOR: Color = Color::from_rgb(0.45, 0.0, 0.0);
    const NORMAL_COLOR: Color = Color::from_rgb(0.29, 0.29, 0.32);
    const JUNCTION_COLOR: Color = Color::from_rgb(0.05, 0.9, 0.8);
    const OUT_COLOR: Color = Color::from_rgb(0.82, 0.05, 0.88);

    pub fn new(file_resource: Arc<RwLock<File>>) -> Self {
        Self {
            file_resource,
            pixel_size: 10f32,
        }
    }

    #[inline]
    pub(crate) fn view(&self) -> Grid<'_, Message, Theme, Renderer> {
        let mut grid = Grid::new();

        let file = self.file_resource.read().unwrap();

        if file.fpga.height() == 0 || file.fpga.width() == 1 {
            return grid;
        }

        let mut direction = true;

        for row in (0..file.fpga.height()).rev() {
            let mut grid_row: GridRow<'_, Message, Theme, Renderer> = GridRow::new();
            for col in 0..file.fpga.width() {
                grid_row = grid_row.push(self.cell(row, col, direction));
            }
            direction = !direction;
            grid = grid.push(grid_row)
        }

        grid
    }

    #[inline]
    pub(crate) fn get_path(&self) -> String {
        let file = self.file_resource.read().unwrap();
        file.get_path().map_or_else(
            || "".to_owned(),
            |path| "-".to_owned() + path.to_str().unwrap_or("Invalid UTF-8 Path"),
        )
    }

    #[inline]
    pub(crate) fn cell(
        &self,
        row: usize,
        col: usize,
        direction: bool,
    ) -> Column<'_, Message, Theme, Renderer> {
        let file = self.file_resource.read().unwrap();

        let cell_data = file
            .get_cell(row, col)
            .expect("Internal Error: cell not found");

        let flags = &cell_data.flags;

        let mut column = Column::new().spacing(0);

        let empty = || self.pixel(Color::TRANSPARENT);

        let row_1 = || self.pixel(Self::NORMAL_COLOR);
        let row_2 = || self.pixel(Self::NORMAL_COLOR);

        let col_1 = || self.not_pixel(CellFlags::NOT_C1, flags);
        let col_2 = || self.not_pixel(CellFlags::NOT_C2, flags);

        let junction = |cell_flag| self.junction_pixel(cell_flag, flags);

        let jc1_r1 = junction(CellFlags::JC1_R1);
        let jc1_r2 = junction(CellFlags::JC1_R2);
        let jc2_r1 = junction(CellFlags::JC2_R1);
        let jc2_r2 = junction(CellFlags::JC2_R2);

        let out = |cell_flag| self.out_pixel(cell_flag, flags);

        let row_1_out = out(CellFlags::R1_OUT);
        let row_2_out = out(CellFlags::R2_OUT);
        let col_1_out = out(CellFlags::C1_OUT);
        let col_2_out = out(CellFlags::C2_OUT);

        let [col_1_order, col_2_order, row_1_order, row_2_order] =
            self.order_pixels(&cell_data.activation_order);

        let mut row = Row::new().spacing(0);

        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_2_out);
        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_1_out);
        row = row.push(empty());
        row = row.push(empty());

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_2());
        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_1());
        row = row.push(empty());
        row = row.push(empty());

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        if direction {
            row = row.push(row_2_out);
            row = row.push(row_2());
            row = row.push(jc2_r2);
            row = row.push(row_2());
            row = row.push(row_2());
            row = row.push(jc1_r2);
            row = row.push(row_2());
            row = row.push(row_2_order);
        } else {
            row = row.push(row_2_order);
            row = row.push(row_2());
            row = row.push(jc2_r2);
            row = row.push(row_2());
            row = row.push(row_2());
            row = row.push(jc1_r2);
            row = row.push(row_2());
            row = row.push(row_2_out);
        }

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_2());
        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_1());
        row = row.push(empty());
        row = row.push(empty());

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_2());
        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_1());
        row = row.push(empty());
        row = row.push(empty());

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        if direction {
            row = row.push(row_1_out);
            row = row.push(row_2());
            row = row.push(jc2_r1);
            row = row.push(row_1());
            row = row.push(row_1());
            row = row.push(jc1_r1);
            row = row.push(row_1());
            row = row.push(row_1_order);
        } else {
            row = row.push(row_1_order);
            row = row.push(row_2());
            row = row.push(jc2_r1);
            row = row.push(row_1());
            row = row.push(row_1());
            row = row.push(jc1_r1);
            row = row.push(row_1());
            row = row.push(row_1_out);
        }

        column = column.push(row);

        let mut row = Row::new().spacing(0);

        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_2());
        row = row.push(empty());
        row = row.push(empty());
        row = row.push(col_1());
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

    fn order_pixels(
        &self,
        activation_order: &ActivationOrder,
    ) -> [Container<'_, Message, Theme, Renderer>; 4] {
        let mut vec: [Container<Message, Theme, Renderer>; 4] =
            std::array::from_fn(|_| self.pixel(Color::TRANSPARENT));

        for (i, selector) in activation_order.into_iter().enumerate() {
            let txt = text(i)
                .size(self.pixel_size * 0.92)
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center);

            vec[selector as usize] = container(txt)
                .width(Length::Fixed(self.pixel_size))
                .height(Length::Fixed(self.pixel_size))
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center)
                .style(|_| container::Style {
                    background: Some(Background::Color(FpgaViewer::NORMAL_COLOR)),
                    ..Default::default()
                });
        }

        vec
    }

    #[inline]
    fn out_pixel(
        &self,
        out: CellFlags,
        cell_flags: &CellFlags,
    ) -> Container<'_, Message, Theme, Renderer> {
        let tmp = if cell_flags.contains(out) {
            FpgaViewer::OUT_COLOR
        } else {
            Color::TRANSPARENT
        };
        self.pixel(tmp)
    }

    #[inline]
    fn not_pixel(
        &self,
        not: CellFlags,
        cell_flags: &CellFlags,
    ) -> Container<'_, Message, Theme, Renderer> {
        let tmp = if cell_flags.contains(not) {
            FpgaViewer::NOT_COLOR
        } else {
            FpgaViewer::NORMAL_COLOR
        };
        self.pixel(tmp)
    }

    #[inline]
    fn junction_pixel(
        &self,
        junction: CellFlags,
        cell_flags: &CellFlags,
    ) -> Container<'_, Message, Theme, Renderer> {
        let tmp = if cell_flags.contains(junction) {
            FpgaViewer::JUNCTION_COLOR
        } else {
            FpgaViewer::NORMAL_COLOR
        };
        self.pixel(tmp)
    }

    #[inline]
    pub fn pixel(&self, color: Color) -> Container<'_, Message, Theme, Renderer> {
        container(Space::new(
            Length::Fixed(self.pixel_size),
            Length::Fixed(self.pixel_size),
        ))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(color)),
            ..container::Style::default()
        })
    }
}
