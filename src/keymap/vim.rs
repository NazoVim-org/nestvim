use crate::editor::Editor;
use crate::keymap::KeymapHandler;

pub struct VimKeymap;

impl VimKeymap {
    pub fn new() -> Self {
        Self
    }
}

impl KeymapHandler for VimKeymap {
    fn handle_key(&self, _editor: &mut Editor) {}

    fn name(&self) -> &'static str {
        "vim"
    }
}