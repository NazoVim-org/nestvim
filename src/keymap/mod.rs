use crate::editor::Editor;
use std::sync::Arc;

mod vim;
pub mod emacs;

pub use vim::VimKeymap;
pub use emacs::EmacsKeymap;
use crossterm::event::{KeyCode, KeyModifiers};

pub trait KeymapHandler: Send + Sync {
    fn handle_key(&self, editor: &mut Editor, key: KeyCode, modifiers: KeyModifiers);
}

pub fn create_keymap(keymap: crate::types::Keymap) -> Arc<dyn KeymapHandler> {
    match keymap {
        crate::types::Keymap::Vim => Arc::new(VimKeymap::new()),
        crate::types::Keymap::Emacs => Arc::new(EmacsKeymap::new()),
    }
}