use crate::editor::Editor;
use crate::keymap::KeymapHandler;
use crossterm::event::{KeyCode, KeyModifiers};

pub struct VimKeymap;

impl VimKeymap {
    pub fn new() -> Self {
        Self
    }
}

impl KeymapHandler for VimKeymap {
    fn handle_key(&mut self, _editor: *mut Editor, _key: KeyCode, _modifiers: KeyModifiers) {}
}
