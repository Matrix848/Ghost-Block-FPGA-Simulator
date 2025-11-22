use crate::gui::fpga_viewer::FpgaViewer;
use crate::io::File;
use iced::widget::{column, container};
use iced::{Element, Fill, Shrink, Size, Task};
use std::string::ToString;
use std::sync::{Arc, RwLock};

pub(crate) mod fpga_viewer;

#[derive(Debug, Clone)]
pub enum Message {}

pub struct GUI {
    title: String,
    fpga_viewer: FpgaViewer,
}

impl GUI {
    const TITLE: &'static str = "Ghost Block FPGA Simulator";

    pub fn new(file_resource: Arc<RwLock<File>>) -> (Self, Task<Message>) {
        (
            Self {
                title: GUI::TITLE.to_string(),
                fpga_viewer: FpgaViewer::new(file_resource),
            },
            Task::none(),
        )
    }

    pub fn run(file_resource: Arc<RwLock<File>>) -> iced::Result {
        iced::application(GUI::title, GUI::update, GUI::view)
            .theme(GUI::theme)
            .window_size(Size::new(1000.0, 600.0))
            .centered()
            .antialiasing(true)
            .run_with(|| GUI::new(file_resource))
    }

    pub fn title(&self) -> String {
        let path_str = self.fpga_viewer.get_path();

        self.title.clone() + &path_str
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {}
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        let main_content = container(
            column![
                container(self.fpga_viewer.view())
                    .height(Shrink)
                    .width(Shrink)
                    .center(Fill)
            ]
            .width(Fill)
            .height(Fill),
        )
        .width(Fill)
        .height(Fill);

        main_content.into()
    }
}
