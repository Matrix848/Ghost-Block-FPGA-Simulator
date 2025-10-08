use crate::io::File;
use crate::ui::menu::menu_bar::button::Status;
use crate::ui::{MENU_FONT_SIZE, Message};
use anyhow::Result;
use iced::widget::{button, text};
use iced::{Border, Color, Element, Length, Task, Theme};
use iced_aw::{Menu, menu::Item, menu_bar, menu_items};

#[derive(Clone, Debug)]
pub(crate) enum MenuMessage {
    ButtonClicked(Label),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Label {
    File,
    New,
    Open,
    Save,
    SaveAs,
    Exit,
}

impl From<Label> for String {
    fn from(value: Label) -> Self {
        match value {
            Label::File => "File".to_string(),
            Label::New => "New".to_string(),
            Label::Open => "Open...".to_string(),
            Label::Save => "Save".to_string(),
            Label::SaveAs => "Save as...".to_string(),
            Label::Exit => "Exit".to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct MenuBar {}

impl MenuBar {
    pub(crate) fn update(&mut self, message: MenuMessage, file: &mut File) -> Task<Message> {
        match message {
            MenuMessage::ButtonClicked(label) => self.on_button_click(label, file),
        }
    }

    pub(crate) fn on_button_click(&mut self, label: Label, file: &mut File) -> Task<Message> {
        match label {
            Label::New => Task::done(Message::NewFileDialog),
            Label::Open => Self::error_check(file.open_file_dialog(), None).0,
            Label::Save => Self::error_check(file.save(), None).0,
            Label::SaveAs => Self::error_check(file.save_as(), None).0,
            Label::Exit => iced::exit(),
            _ => Task::none(),
        }
    }

    //TODO: remove [message] field if not needed.
    #[inline]
    fn error_check<T>(result: Result<T>, message: Option<Message>) -> (Task<Message>, Option<T>) {
        match result {
            Ok(value) => (
                message.map_or_else(|| Task::none(), |message| Task::done(message)),
                Some(value),
            ),
            Err(e) => (Task::done(Message::ErrorDialog(e.to_string())), None),
        }
    }

    #[inline]
    pub(crate) fn view(&self) -> Element<'_, Message> {
        let tpl = |items| Menu::new(items).max_width(100.).offset(15.0).spacing(5.0);

        #[rustfmt::skip]
        let mb = menu_bar!(
            (
                menu_button(&Label::File).style(flat_button_style),
                tpl(menu_items!(
                    (action_button(&Label::New))
                    (action_button(&Label::Open))
                    (action_button(&Label::Save))
                    (action_button(&Label::SaveAs))
                    (action_button(&Label::Exit))
                ))
            )
        );

        mb.into()
    }
}

#[inline]
pub fn menu_button(label: &Label) -> button::Button<'_, Message, Theme, iced::Renderer> {
    button(text(String::from(*label)).size(MENU_FONT_SIZE))
        .padding([6, 8])
        .style(flat_button_style)
}

#[inline]
pub fn action_button(label: &Label) -> button::Button<'_, Message, Theme, iced::Renderer> {
    button(text(String::from(*label)).size(MENU_FONT_SIZE))
        .padding([4, 8])
        .on_press(Message::MenuMessage(MenuMessage::ButtonClicked(*label)))
        .width(Length::Fill)
        .style(flat_button_style)
}

#[inline]
pub fn flat_button_style(theme: &Theme, status: Status) -> button::Style {
    let ext_palette = theme.extended_palette();
    let base = button::Style {
        text_color: ext_palette.background.base.text,
        border: Border::default().rounded(3.0),
        ..button::Style::default()
    };

    match status {
        Status::Active => base.with_background(Color::TRANSPARENT),
        Status::Hovered => base.with_background(ext_palette.background.weak.color),
        Status::Disabled => base.with_background(ext_palette.background.base.color),
        Status::Pressed => base.with_background(ext_palette.background.weak.color),
    }
}
