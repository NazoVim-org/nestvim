use crate::editor::Editor;
use std::cell::RefCell;
use std::rc::Rc;

mod vim;
pub mod emacs;

pub use vim::VimKeymap;
pub use emacs::EmacsKeymap;
use crossterm::event::{KeyCode, KeyModifiers};

pub trait KeymapHandler: Send + Sync {
    fn handle_key(&mut self, editor: *mut Editor, key: KeyCode, modifiers: KeyModifiers);
}

pub fn create_keymap(keymap: crate::types::Keymap) -> Rc<RefCell<dyn KeymapHandler>> {
    match keymap {
        crate::types::Keymap::Vim => Rc::new(RefCell::new(VimKeymap::new())),
        crate::types::Keymap::Emacs => Rc::new(RefCell::new(EmacsKeymap::new())),
    }
}
