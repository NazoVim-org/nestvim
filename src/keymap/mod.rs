use crate::editor::Editor;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

pub mod emacs;
mod vim;

use crossterm::event::{KeyCode, KeyModifiers};
pub use emacs::EmacsKeymap;
pub use vim::VimKeymap;

pub trait KeymapHandler: Send + Sync {
    fn handle_key<'a>(
        &'a mut self,
        editor: &'a mut Editor,
        key: KeyCode,
        modifiers: KeyModifiers,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
}

pub fn create_keymap(keymap: crate::types::Keymap) -> Rc<RefCell<dyn KeymapHandler>> {
    match keymap {
        crate::types::Keymap::Vim => Rc::new(RefCell::new(VimKeymap::new())),
        crate::types::Keymap::Emacs => Rc::new(RefCell::new(EmacsKeymap::new())),
    }
}
