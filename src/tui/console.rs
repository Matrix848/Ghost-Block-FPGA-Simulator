use crate::tui::Command;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, ToLine};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

#[derive(Debug)]
pub(super) struct Console {
    // List of the old lines(both commands and input).
    lines: Vec<Line<'static>>,
    // History of KNOWN commands written in the console.
    history: Vec<String>,
    // History of KNOWN commands written in the console.
    history_idx: usize,
    // Current input being typed.
    input: String,
    // Cursor position in the input.
    cursor_position: usize,
    // Scroll offset for viewing history.
    scroll_offset: usize,
    // Number of lines in the console.
    visible_height: usize,
    // The text before the prompt.
    ps1: String,
}

impl Default for Console {
    fn default() -> Self {
        Self {
            lines: Vec::default(),
            history: vec![String::from("")],
            history_idx: 0,
            input: String::default(),
            cursor_position: 0,
            scroll_offset: 0,
            visible_height: 0,
            ps1: String::from(""),
        }
    }
}

impl Console {
    const PROMPT: &'static str = " > ";
    const SPACING: &'static str = " ";

    #[inline]
    pub(super) fn add_line(&mut self, line: Line<'static>) {
        self.scroll_offset = 0;

        self.lines.push(line);
    }

    #[inline]
    #[must_use]
    pub(super) fn submit_input(&mut self) -> Option<Command> {
        let command_line = self.input.clone();

        self.input.clear();
        self.cursor_position = 0;
        self.history_idx = 0;

        self.add_line(self.format_command_line(command_line.clone()));
        let command = self.handle_command(&command_line);
        command
    }

    #[inline]
    pub(super) fn insert_char(&mut self, c: char) {
        self.history_idx = 0;
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    #[inline]
    pub(super) fn delete_char(&mut self) {
        self.history_idx = 0;
        if self.cursor_position > 0 {
            self.input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    #[inline]
    pub(super) fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    #[inline]
    pub(super) fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    #[inline]
    pub(super) fn scroll_up(&mut self) {
        if self.scroll_offset < self.lines.len().saturating_sub(self.visible_height - 1) {
            self.scroll_offset += 1;
        }
    }

    #[inline]
    pub(super) fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    #[inline]
    fn history_to_input(&mut self, index: usize) {
        self.input = self.history[self.history.len() - 1 - index].clone();
        self.cursor_position = self.input.len()
    }

    #[inline]
    pub(super) fn history_back(&mut self) {
        if self.history_idx == 0 {
            self.history[0] = self.input.clone();
        }
        if self.history_idx < self.history.len() - 1 {
            self.history_idx += 1;
            self.history_to_input(self.history_idx - 1)
        }
    }

    #[inline]
    pub(super) fn history_forth(&mut self) {
        if self.history_idx == 1 {
            self.history_to_input(self.history.len() - 1)
        } else {
            self.history_idx = self.history_idx.saturating_sub(1);
            self.history_to_input(self.history_idx - 1)
        }
    }

    #[inline]
    fn format_command_line(&self, line: String) -> Line<'static> {
        Line::from(format!("{}{}{}", self.ps1, Self::PROMPT, line))
    }

    #[inline]
    fn format_spacing(line: String) -> String {
        format!("{}{}", Self::SPACING, line)
    }

    #[inline]
    fn push_to_history(&mut self, command_line: &str) {
        self.history.push(String::from(command_line));
    }

    #[inline]
    pub(super) fn handle_command(&mut self, mut command_line: &str) -> Option<Command> {
        command_line = command_line.trim_start();
        let command: Vec<&str> = command_line.split(" ").collect();

        if command[0].eq("") {
            return None;
        }
        self.push_to_history(command_line);

        match command[0] {
            "quit" => Some(Command::Quit),
            "exit" => Some(Command::Quit),
            "clear" => {
                self.lines.clear();
                None
            }
            str => {
                let unknown_string = Self::format_spacing("Unknown command: ".to_string());
                let line = Line::<'static>::from(vec![
                    Span::styled(unknown_string, Style::default().fg(Color::Red)),
                    Span::styled(str.to_string(), Style::default().fg(Color::White)),
                ])
                .style(Style::default().fg(Color::Red));

                self.add_line(line);

                None
            }
        }
    }

    pub(super) fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Console");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        self.visible_height = inner.height as usize;

        let mut lines = self.lines.clone();

        let input_line = self.format_command_line(self.input.clone());

        lines.push(input_line.to_line());

        let total_lines = lines.len();

        let end_idx = total_lines
            .saturating_sub(self.scroll_offset)
            .max(self.visible_height.min(total_lines));

        let start_idx = total_lines.saturating_sub(self.scroll_offset + self.visible_height);

        let visible_lines = lines[start_idx..end_idx].to_vec();

        let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });

        frame.render_widget(paragraph, inner);

        // Calculate cursor position
        // The cursor should only be visible when we're scrolled to the bottom (scroll_offset == 0)
        if self.scroll_offset == 0 {
            // Calculate which line the input is on (relative to visible area)
            let input_line_absolute = total_lines.saturating_sub(1); // Last line
            let input_line_relative = input_line_absolute.saturating_sub(start_idx);

            // Only set cursor if the input line is actually visible
            if input_line_relative < self.visible_height {
                let cursor_x =
                    inner.x + Self::PROMPT.chars().count() as u16 + self.cursor_position as u16;
                let cursor_y = inner.y + input_line_relative as u16;

                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }
}
