use crate::buffer::TextBuffer;
use crate::terminal::Terminal;
use crate::types::{EditorState, Mode};

pub struct Renderer {
    scroll_top: usize,
}

impl Renderer {
    pub fn new() -> Self {
        Self { scroll_top: 1 }
    }

    pub fn render(&mut self, terminal: &Terminal, buffer: &TextBuffer, state: &EditorState) {
        let rows = terminal.rows() as usize;
        let visible_rows = rows.saturating_sub(2); // Status bar

        // Adjust scroll position
        if state.cursor.line < self.scroll_top {
            self.scroll_top = state.cursor.line;
        } else if state.cursor.line >= self.scroll_top + visible_rows {
            self.scroll_top = state.cursor.line - visible_rows + 1;
        }

        // Clear screen
        let _ = terminal.clear_screen();

        // Draw lines
        for i in 0..visible_rows {
            let buf_line = self.scroll_top + i;
            let raw_line = buffer.get_line(buf_line);
            let display_text = if raw_line.is_empty() && buf_line > buffer.line_count() {
                "~".to_string()
            } else {
                raw_line
            };

            let _ = terminal.write_line((i + 1) as u16, &display_text);
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
        let _ = terminal.write_status(&status);

        // Move cursor
        let screen_row = (state.cursor.line.saturating_sub(self.scroll_top) + 1) as u16;
        let _ = terminal.move_cursor(screen_row, (state.cursor.col + 1) as u16);
        let _ = terminal.flush();
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
