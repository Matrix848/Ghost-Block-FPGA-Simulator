use std::io::Result;
mod tui;
mod viewer;

fn main() -> Result<()> {
    better_panic::install();
    // Takes control of the terminal
    let terminal = ratatui::init();

    let mut app = tui::TUI::default();

    let result = app.run(terminal);

    // Restores the control to the terminal
    ratatui::restore();

    result
}
