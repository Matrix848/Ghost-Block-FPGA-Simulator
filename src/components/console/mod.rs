use super::Component;
use crate::action::Action;
use crate::simulator_core::FPGA;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{prelude::*, widgets::*};
use std::fs;
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::UnboundedSender;

mod commands;

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Processing,
}

#[derive(Default)]
pub struct Console {
    // Transmission channel.
    command_tx: Option<UnboundedSender<Action>>,
    // Current mode.
    mode: Mode,
    // Title.
    title: String,
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
    // Working path.
    path: Option<PathBuf>,
    // Working path.
    fpga: Option<Arc<RwLock<FPGA>>>,
    // Selected cell.
    selected_cell: Option<(usize, usize)>,
}

impl Console {
    pub const PROMPT: &'static str = ">";

    // ASCII only
    const PROMPT_LEN: usize = Self::PROMPT.len();
    // Use this for Unicode compatibility(you need to set it by hand)
    // const PROMPT_LEN: usize = 1;

    pub fn new() -> Self {
        Self {
            command_tx: Option::default(),
            mode: Mode::Normal,
            title: String::from("Console"),
            lines: Vec::default(),
            history: vec![String::from("")],
            history_idx: 0,
            input: String::default(),
            cursor_position: 0,
            scroll_offset: 0,
            visible_height: 0,
            ps1: String::from(""),
            path: None,
            fpga: None,
            selected_cell: None,
        }
    }

    pub(super) fn submit_input(&mut self) {
        // Copies the input and clears it.
        let command_line = self.input.clone();
        self.input.clear();

        // Resets cursor position and history index to default.
        self.cursor_position = 0;
        self.history_idx = 0;

        // Add the current line to the lines of the console.
        self.add_line(self.format_command_line(command_line.clone()));

        if command_line.is_empty() {
            return;
        }

        self.push_to_history(&command_line);

        // Removes leading whitespaces.
        let command_line = command_line.trim_start();

        // Splits the command into the command and it's arguments.
        let mut parts = command_line.split(" ");
        let command = parts.next().unwrap();
        let args: Vec<&str> = parts.collect();

        self.handle_command(command, args)
    }

    fn handle_command(&mut self, command: &str, args: Vec<&str>) {
        match command {
            "quit" => self.command_tx.clone().unwrap().send(Action::Quit).unwrap(),
            "clear" => self.lines.clear(),
            "open" => {
                self.open(args);
            }
            "save" => {}
            "sel" | "select" => (),
            str => {
                self.unknown_command(str.to_string());
            }
        }
    }

    fn open(&mut self, args: Vec<&str>) {
        #[inline]
        #[must_use]
        fn help() -> Vec<Span<'static>> {
            vec![
                "Usage: ".into(),
                "open ".cyan(),
                "<".into(),
                "path/to/file".yellow(),
                ">".into(),
            ]
        }

        match args.len() {
            0 => {
                let mut spans = Vec::<Span>::with_capacity(7);

                spans.push("Error: ".red());
                spans.push("no path specified.".into());
                spans.append(&mut help());

                self.add_line(spans.into());
            }
            1 => match args[0] {
                "--help" => self.add_line(help().into()),
                option if option.starts_with("--") => {
                    let mut spans = Vec::<Span>::with_capacity(10);

                    spans.push("Error: ".red());
                    spans.push("invalid option '".into());
                    spans.push(option.to_string().gray());
                    spans.push("'.".into());
                    spans.append(&mut help());

                    self.add_line(spans.into());
                }
                str => {
                    let path = Path::new(str);

                    let file = fs::read(path);

                    if file.is_err() {
                        self.add_line(
                            vec![
                                "Error reading file: ".red(),
                                "path '".into(),
                                str.to_string().gray(),
                                "' does not exist.".into(),
                            ]
                            .into(),
                        );
                        return;
                    }

                    let fpga: postcard::Result<FPGA> = postcard::from_bytes(&file.unwrap());

                    if let Err(error) = fpga {
                        self.add_line(
                            vec![
                                "Error deserializing file:".red(),
                                format!("{:?}", error).gray(),
                            ]
                            .into(),
                        );
                        return;
                    }

                    self.fpga = Some(Arc::new(RwLock::new(fpga.unwrap())));
                    self.command_tx
                        .clone()
                        .unwrap()
                        .send(Action::Open(path.into()))
                        .unwrap();
                }
            },
            _ => {
                let mut spans = Vec::<Span>::with_capacity(7);

                spans.push("Error: ".red());
                spans.push("too many arguments.".into());
                spans.append(&mut help());

                self.add_line(spans.into());
            }
        }
    }

    fn unknown_command(&mut self, value: String) {
        // Formats the string so that the only "Unknown command: " is red.
        let line = Line::<'static>::from(vec!["Unknown command: ".red(), value.into()]);

        // Adds the error line to the lines of the console
        self.add_line(line);
    }

    #[inline]
    fn format_command_line(&self, line: impl ToString) -> Line<'static> {
        Line::from(format!("{}{}{}", self.ps1, Self::PROMPT, line.to_string()))
    }

    #[inline]
    fn push_to_history(&mut self, command_line: &str) {
        self.history.push(String::from(command_line));
    }

    #[inline]
    pub(super) fn add_line(&mut self, line: Line<'static>) {
        self.scroll_offset = 0;

        self.lines.push(line);
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
            self.history_idx = self.history_idx.saturating_sub(1);
            self.history_to_input(self.history.len() - 1);
        } else if self.history_idx > 0 {
            self.history_idx = self.history_idx.saturating_sub(1);
            self.history_to_input(self.history_idx - 1)
        }
    }
}

impl Component for Console {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }
    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        // If the console is processing the only event that should be processed is ctrl+c.
        if self.mode == Mode::Normal {
            match (key.kind, key.code) {
                (KeyEventKind::Press, KeyCode::Char(c)) => {
                    self.insert_char(c);
                    return Ok(None);
                }
                (KeyEventKind::Press, KeyCode::Backspace) => {
                    self.delete_char();
                    return Ok(None);
                }
                (KeyEventKind::Press, KeyCode::Enter) => {
                    self.submit_input();
                    return Ok(None);
                }
                (KeyEventKind::Press, KeyCode::Left) => {
                    self.move_cursor_left();
                    return Ok(None);
                }
                (KeyEventKind::Press, KeyCode::Right) => {
                    self.move_cursor_right();
                    return Ok(None);
                }
                (KeyEventKind::Press, KeyCode::Up) => {
                    self.history_back();
                    return Ok(None);
                }
                (KeyEventKind::Press, KeyCode::Down) => {
                    self.history_forth();
                    return Ok(None);
                }
                _ => (),
            }
        } else if key.kind == KeyEventKind::Press
            && (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('C'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            return Ok(Some(Action::InterruptProcessing));
        }
        Ok(None)
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> color_eyre::Result<Option<Action>> {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.scroll_up();
                Ok(None)
            }
            MouseEventKind::ScrollDown => {
                self.scroll_down();
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.clone());
        let text_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
            .split(area)[1];

        let inner = block.inner(text_area);
        frame.render_widget(block, text_area);

        self.visible_height = inner.height as usize;
        let line_width = inner.width as usize;

        // Wrap all lines to get true visual line count
        let mut wrapped_lines: Vec<Line> = Vec::new();
        let mut visual_line_mapping: Vec<usize> = Vec::new(); // Maps visual line to original line index

        // Wrap history lines
        for (idx, line) in self.lines.iter().enumerate() {
            let text = line.to_string(); // Get the text content
            let chunks = wrap_text(&text, line_width);
            for chunk in chunks {
                wrapped_lines.push(Line::from(chunk));
                visual_line_mapping.push(idx);
            }
        }

        // Wrap and add input line
        let input_text = format!("{}{}", Self::PROMPT, self.input);
        let input_chunks = wrap_text(&input_text, line_width);
        let input_start_visual_line = wrapped_lines.len();

        for chunk in input_chunks {
            wrapped_lines.push(Line::from(chunk));
            visual_line_mapping.push(self.lines.len());
        }

        // Calculate visible range based on actual visual lines
        let total_visual_lines = wrapped_lines.len();
        let end_idx = total_visual_lines.saturating_sub(self.scroll_offset);
        let start_idx = end_idx.saturating_sub(self.visible_height);

        let visible_lines = wrapped_lines[start_idx..end_idx].to_vec();

        // Use Wrap::NoWrap since we've already wrapped manually
        let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);

        // Calculate cursor position with proper wrapping
        if self.scroll_offset == 0 {
            let cursor_pos_in_full = Self::PROMPT_LEN + self.cursor_position;

            // Which wrapped line is the cursor on?
            let wrapped_line_idx = cursor_pos_in_full / line_width;
            let pos_in_wrapped_line = cursor_pos_in_full % line_width;

            // Find the visual line number for the input's first wrapped line
            let input_visual_line = input_start_visual_line + wrapped_line_idx;

            // Check if this visual line is in the visible range
            if input_visual_line >= start_idx && input_visual_line < end_idx {
                let cursor_y_offset = input_visual_line - start_idx;

                let cursor_x = inner.x + pos_in_wrapped_line as u16;
                let cursor_y = inner.y + cursor_y_offset as u16;

                if cursor_y < inner.y + inner.height {}
            }
        }

        Ok(())
    }
}

// Helper function to wrap text at character boundaries
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for ch in text.chars() {
        let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);

        if current_width + char_width > width {
            // Line is full, push it and start a new one
            if !current_line.is_empty() {
                result.push(current_line);
                current_line = String::new();
                current_width = 0;
            }
        }

        current_line.push(ch);
        current_width += char_width;
    }

    // Push the last line
    if !current_line.is_empty() {
        result.push(current_line);
    }

    // If text was empty, return at least one empty line
    if result.is_empty() {
        result.push(String::new());
    }

    result
}
