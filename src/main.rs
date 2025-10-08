use crate::ui::GUI;
use iced::Size;
mod io;
mod ui;

fn main() -> iced::Result {
    iced::application(GUI::title, GUI::update, GUI::view)
        .theme(GUI::theme)
        .window_size(Size::new(1000.0, 600.0))
        .centered()
        .antialiasing(true)
        .run()
}
