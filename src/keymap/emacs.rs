use crate::editor::Editor;
use crate::keymap::KeymapHandler;
use crossterm::event::{KeyCode, KeyModifiers};

pub struct EmacsKeymap;

impl EmacsKeymap {
    pub fn new() -> Self {
        Self
    }
}

impl KeymapHandler for EmacsKeymap {
    fn handle_key(&self, editor: &mut Editor) {}

    fn name(&self) -> &'static str {
        "emacs"
    }
}

pub async fn handle_emacs_key(editor: &mut Editor, key: KeyCode, modifiers: KeyModifiers) {
    use crossterm::event::KeyModifiers;
    
    let has_ctrl = modifiers.contains(KeyModifiers::CONTROL);

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
        (false, KeyCode::Char(c)) => {
            editor.insert_char(c);
        }
        (false, KeyCode::Backspace) => {
            editor.delete_char_backward();
        }
        (false, KeyCode::Enter) => {
            editor.insert_newline();
        }
        _ => {}
    }
}