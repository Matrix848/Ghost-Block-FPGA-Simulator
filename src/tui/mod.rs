use crate::tui::console::Console;
use ratatui::crossterm::event::{
    EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent,
    MouseEventKind,
};
use ratatui::crossterm::{event, execute};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders};
use ratatui::{DefaultTerminal, Frame};
use simulator_core::FPGA;
use std::io::Result;
use std::path::PathBuf;

mod console;

#[derive(Debug, Default, PartialEq, Eq)]
#[repr(u8)]
enum RunningState {
    #[default]
    Running,
    Quit,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
enum Command {
    Quit,
}

#[derive(Debug, Default)]
pub struct TUI {
    state: RunningState,
    path: Option<PathBuf>,
    fpga: Option<FPGA>,
    console: Console,
}

impl TUI {
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let result = Ok(());

        execute!(std::io::stdout(), EnableMouseCapture)?;

        while self.state == RunningState::Running {
            match event::read()? {
                // Input handling
                Event::Key(key_event) => self.handle_key_event(key_event),
                Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
                _ => (),
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        result
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) {
        match event.kind {
            MouseEventKind::ScrollUp => self.console.scroll_up(),
            MouseEventKind::ScrollDown => self.console.scroll_down(),
            _ => (),
        }
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        match (event.kind, event.code) {
            (KeyEventKind::Press, KeyCode::Char('q')) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.state = RunningState::Quit
                } else {
                    self.console.insert_char('q');
                }
            }

            // Console events:
            (KeyEventKind::Press, KeyCode::Char(c)) => {
                self.console.insert_char(c);
            }
            (KeyEventKind::Press, KeyCode::Backspace) => {
                self.console.delete_char();
            }
            (KeyEventKind::Press, KeyCode::Enter) => {
                let result = self.console.submit_input();
                if let Some(command) = result {
                    self.handle_commands(command)
                }
            }
            (KeyEventKind::Press, KeyCode::Left) => {
                self.console.move_cursor_left();
            }
            (KeyEventKind::Press, KeyCode::Right) => {
                self.console.move_cursor_right();
            }
            (KeyEventKind::Press, KeyCode::Up) => {
                self.console.history_back();
            }
            (KeyEventKind::Press, KeyCode::Down) => {
                self.console.history_forth();
            }
            _ => (),
        }
    }

    fn handle_commands(&mut self, command: Command) {
        match command {
            Command::Quit => self.state = RunningState::Quit,
        }
    }

    #[inline]
    fn draw(&mut self, frame: &mut Frame) {
        let vertical_layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(frame.area());
        frame.render_widget(Block::new().borders(Borders::ALL), vertical_layout[1]);

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(vec![Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(vertical_layout[0]);
        frame.render_widget(
            Block::new().borders(Borders::ALL).title("FPGA"),
            main_layout[0],
        );

        self.console.draw(frame, main_layout[1]);
    }
}
