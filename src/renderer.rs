use crate::buffer::TextBuffer;
use crate::terminal::Terminal;
use crate::types::{EditorState, Mode};
use std::io;

pub struct Renderer {
    scroll_top: usize,
}

impl Renderer {
    pub fn new() -> Self {
        Self { scroll_top: 1 }
    }

    pub fn render(&mut self, terminal: &mut Terminal, buffer: &TextBuffer, state: &EditorState) -> io::Result<()> {
        let rows = terminal.rows().max(2);
        let visible_rows = (rows as usize).saturating_sub(2).max(1);

        // Adjust scroll position
        if state.cursor.line < self.scroll_top {
            self.scroll_top = state.cursor.line;
        } else if state.cursor.line >= self.scroll_top + visible_rows {
            self.scroll_top = state.cursor.line - visible_rows + 1;
        }

        // Clear screen
        terminal.clear_screen()?;

        // Draw lines
        for i in 0..visible_rows {
            let buf_line = self.scroll_top + i;
            let raw_line = buffer.get_line(buf_line);
            let display_text = if raw_line.is_empty() && buf_line > buffer.line_count() && buffer.line_count() > 0 {
                "~".to_string()
            } else {
                raw_line
            };

            terminal.write_line((i + 1) as u16, &display_text)?;
        }

        // Status line
        let status = if state.mode == Mode::Command {
            format!(":{}", state.command_buffer)
        } else {
            format!(
                "-- {} -- {} {}",
                state.mode,
                state.file_path.as_ref().map_or("[No Name]", |p| p.to_str().unwrap_or("[Invalid Path]")),
                if state.dirty { "[+]" } else { "" }
            )
        };
        terminal.write_status(&status)?;

        // Move cursor
        let screen_row = (state.cursor.line.saturating_sub(self.scroll_top) + 1) as u16;
        terminal.move_cursor(screen_row, (state.cursor.col + 1) as u16)?;
        terminal.flush()
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
