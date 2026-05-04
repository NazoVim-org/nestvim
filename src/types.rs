use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Command => write!(f, "COMMAND"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct EditorState {
    pub mode: Mode,
    pub cursor: Position,
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            mode: Mode::Normal,
            cursor: Position { line: 1, col: 0 },
            file_path: None,
            dirty: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    ModeChange { from: Mode, to: Mode },
    BufferChange,
    Key { mode: Mode, key: char },
    BufferSave { file_path: Option<PathBuf> },
}
