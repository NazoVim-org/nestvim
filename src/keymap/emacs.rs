use crate::editor::Editor;
use crate::keymap::KeymapHandler;
use crate::types::PluginEvent;
use crossterm::event::{KeyCode, KeyModifiers};

pub struct EmacsKeymap;

impl EmacsKeymap {
    pub fn new() -> Self {
        Self
    }
}

impl KeymapHandler for EmacsKeymap {
    fn handle_key(&self, editor: &mut Editor, key: KeyCode, modifiers: KeyModifiers) {
        use crossterm::event::KeyModifiers;

        let has_ctrl = modifiers.contains(KeyModifiers::CONTROL);

        if let KeyCode::Char(c) = key {
            editor.plugin_manager.emit(PluginEvent::Key {
                mode: editor.state.mode,
                key: c,
            });
        }

        match (has_ctrl, key) {
            (true, KeyCode::Char('f')) => {
                editor.cursor_right(1);
            }
            (true, KeyCode::Char('b')) => {
                editor.cursor_left(1);
            }
            (true, KeyCode::Char('n')) => {
                editor.cursor_down(1);
            }
            (true, KeyCode::Char('p')) => {
                editor.cursor_up(1);
            }
            (true, KeyCode::Char('a')) => {
                editor.cursor_line_start();
            }
            (true, KeyCode::Char('e')) => {
                editor.cursor_line_end();
            }
            (true, KeyCode::Char('d')) => {
                editor.delete_char_forward();
            }
            (true, KeyCode::Char('k')) => {
                editor.kill_line();
            }
            (true, KeyCode::Char('h')) => {
                editor.delete_char_backward();
            }
            (true, KeyCode::Char('w')) => {
                editor.kill_word();
            }
            (true, KeyCode::Char('y')) => {
                editor.yank_pop();
            }
            (true, KeyCode::Char('t')) => {
                editor.transpose_chars();
            }
            (true, KeyCode::Char('v')) => {
                editor.scroll_up_one();
            }
            (true, KeyCode::Char('l')) => {
                editor.clear_screen();
                editor.scroll_cursor_to_center();
            }
            (true, KeyCode::Char('g')) => {
                editor.abort();
            }
            (false, KeyCode::Char(c)) if !c.is_control() => {
                editor.insert_char(c);
            }
            (false, KeyCode::Backspace) => {
                editor.delete_char_backward();
            }
            (false, KeyCode::Delete) => {
                editor.delete_char_forward();
            }
            (false, KeyCode::Enter) => {
                editor.insert_newline();
            }
            (false, KeyCode::Tab) => {
                editor.insert_tab();
            }
            (false, KeyCode::Up) => {
                editor.cursor_up(1);
            }
            (false, KeyCode::Down) => {
                editor.cursor_down(1);
            }
            (false, KeyCode::Left) => {
                editor.cursor_left(1);
            }
            (false, KeyCode::Right) => {
                editor.cursor_right(1);
            }
            (false, KeyCode::Home) => {
                editor.cursor_line_start();
            }
            (false, KeyCode::End) => {
                editor.cursor_line_end();
            }
            (false, KeyCode::PageUp) => {
                editor.scroll_up();
            }
            (false, KeyCode::PageDown) => {
                editor.scroll_down();
            }
            _ => {}
        }
    }
}