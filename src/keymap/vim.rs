use crate::editor::Editor;
use crate::keymap::KeymapHandler;
use crate::types::Mode;
use crossterm::event::{KeyCode, KeyModifiers};

pub struct VimKeymap;

impl VimKeymap {
    pub fn new() -> Self {
        Self
    }
}

impl KeymapHandler for VimKeymap {
    fn handle_key(&mut self, editor: *mut Editor, key: KeyCode, modifiers: KeyModifiers) {
        let runtime = tokio::runtime::Handle::current();
        runtime.block_on(async {
            // SAFETY: editor pointer is created from &mut self in Editor::handle_key.
            let editor = unsafe { &mut *editor };
            editor.vim_on_key_event(key);

            if modifiers.contains(KeyModifiers::CONTROL) {
                match editor.state.mode {
                    Mode::Normal | Mode::Insert | Mode::Replace => match key {
                        KeyCode::Char('d') => {
                            editor.scroll_by(editor.terminal.rows() as usize / 2);
                            editor.needs_render = true;
                            return;
                        }
                        KeyCode::Char('u') => {
                            editor.scroll_by(editor.terminal.rows() as usize / 2);
                            editor.needs_render = true;
                            return;
                        }
                        KeyCode::Char('y') => {
                            editor.scroll_up_one();
                            editor.needs_render = true;
                            return;
                        }
                        KeyCode::Char('e') => {
                            editor.scroll_down_one();
                            editor.needs_render = true;
                            return;
                        }
                        KeyCode::Char('g') => {
                            editor.show_file_info();
                            editor.needs_render = true;
                            return;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            match editor.state.mode {
                Mode::Normal => editor.handle_normal(key).await,
                Mode::Insert => editor.handle_insert(key).await,
                Mode::Command => editor.handle_command(key).await,
                Mode::Visual => editor.handle_visual(key).await,
                Mode::Replace => editor.handle_replace(key).await,
            }
        });
    }
}
