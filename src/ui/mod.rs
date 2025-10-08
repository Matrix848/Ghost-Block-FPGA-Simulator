use crate::ui::fpga_viewer::FpgaViewer;
use crate::ui::menu::menu_bar::{MenuBar, MenuMessage};
use iced::widget::{button, column, container, row, stack, text, text_input};
use iced::{Background, Border, Center, Color, Element, Fill, Length, Shrink, Task};
use simulator_core::FPGA;
use std::string::ToString;

pub(crate) mod menu {
    pub(crate) mod menu_bar;
}

pub(crate) mod fpga_viewer;

static MENU_FONT_SIZE: f32 = 14f32;

#[derive(Debug, Clone)]
pub enum Message {
    MenuMessage(MenuMessage),
    ErrorDialog(String),
    NewFileDialog,
    NewFile(usize, usize),
    ModalWidthInput(String),
    ModalHeightInput(String),
    ModalConfirm,
    ModalCancel,
}

pub struct GUI {
    title: String,
    fpga_viewer: FpgaViewer,
    menu_bar: MenuBar,
    show_new_file_modal: bool,
    modal_width_input: String,
    modal_height_input: String,
}

impl Default for GUI {
    fn default() -> Self {
        Self {
            title: GUI::TITLE.to_string(),
            fpga_viewer: Default::default(),
            menu_bar: Default::default(),
            show_new_file_modal: false,
            modal_height_input: Default::default(),
            modal_width_input: Default::default(),
        }
    }
}

impl GUI {
    const TITLE: &'static str = "Ghost Block FPGA Simulator";

    pub fn title(&self) -> String {
        let path_str = self.fpga_viewer.file.get_path().map_or_else(
            || "".to_owned(),
            |path| "-".to_owned() + path.to_str().unwrap_or("Invalid UTF-8 Path"),
        );

        self.title.clone() + &path_str
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MenuMessage(menu_message) => self
                .menu_bar
                .update(menu_message, &mut self.fpga_viewer.file),
            Message::ErrorDialog(_str) => Task::none(),
            Message::NewFileDialog => {
                self.show_new_file_modal = true;
                self.modal_width_input.clear();
                self.modal_height_input.clear();
                Task::none()
            }
            Message::NewFile(width, height) => {
                self.fpga_viewer.file.set_path(None);
                self.fpga_viewer.file.fpga = FPGA::new(width, height);
                Task::none()
            }
            Message::ModalWidthInput(value) => {
                self.modal_width_input = value;
                Task::none()
            }
            Message::ModalHeightInput(value) => {
                self.modal_height_input = value;
                Task::none()
            }
            Message::ModalConfirm => {
                self.show_new_file_modal = false;

                let width = self.modal_width_input.parse::<usize>().unwrap_or(10);
                let height = self.modal_height_input.parse::<usize>().unwrap_or(10);

                Task::done(Message::NewFile(width, height))
            }
            Message::ModalCancel => {
                self.show_new_file_modal = false;
                Task::none()
            }
        }
    }

    fn new_file_modal(&self) -> Element<'_, Message> {
        let width_input = text_input("Width", &self.modal_width_input)
            .on_input(Message::ModalWidthInput)
            .padding(10);

        let height_input = text_input("Height", &self.modal_height_input)
            .on_input(Message::ModalHeightInput)
            .padding(10);

        let confirm_button = button(text("Create").align_x(iced::alignment::Horizontal::Center))
            .on_press(Message::ModalConfirm)
            .padding(10)
            .width(Length::Fixed(100.0));

        let cancel_button = button(text("Cancel").align_x(iced::alignment::Horizontal::Center))
            .on_press(Message::ModalCancel)
            .padding(10)
            .width(Length::Fixed(100.0));

        let dialog_content = column![
            text("Create New FPGA").size(20),
            column![text("Width:").size(14), width_input].spacing(5),
            column![text("Height:").size(14), height_input].spacing(5),
            row![confirm_button, cancel_button]
                .spacing(10)
                .align_y(Center),
        ]
        .spacing(15)
        .padding(20)
        .align_x(Center);

        let dialog = container(dialog_content)
            .width(Length::Fixed(350.0))
            .padding(20)
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                border: Border {
                    color: Color::from_rgb(0.4, 0.4, 0.4),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            });

        let backdrop = button(container("").width(Fill).height(Fill).style(
            |_theme: &iced::Theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
                ..Default::default()
            },
        ))
        .on_press(Message::ModalCancel)
        .padding(0)
        .style(|_theme: &iced::Theme, _status| button::Style {
            background: None,
            ..Default::default()
        });

        // Center the dialog on top of the backdrop
        container(stack![
            backdrop,
            container(dialog)
                .width(Fill)
                .height(Fill)
                .align_x(Center)
                .align_y(Center)
        ])
        .width(Fill)
        .height(Fill)
        .into()
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        let mb = self.menu_bar.view();
        let main_content = container(
            column![
                mb,
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

        if self.show_new_file_modal {
            stack![main_content, self.new_file_modal()].into()
        } else {
            main_content.into()
        }
    }
}
