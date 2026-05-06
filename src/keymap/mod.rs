use crate::editor::Editor;
use crate::types::Keymap;
use std::sync::Arc;

mod vim;
pub mod emacs;

pub use vim::VimKeymap;
pub use emacs::EmacsKeymap;

pub fn create_keymap(keymap: Keymap) -> Arc<dyn KeymapHandler + Send + Sync> {
    match keymap {
        Keymap::Vim => Arc::new(VimKeymap::new()),
        Keymap::Emacs => Arc::new(EmacsKeymap::new()),
    }
}

pub trait KeymapHandler: Send + Sync {
    fn handle_key(&self, editor: &mut Editor);
    fn name(&self) -> &'static str;
}