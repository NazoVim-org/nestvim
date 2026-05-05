use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum NestvimError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("Terminal error: {0}")]
    Terminal(String),
    #[error("No file path set")]
    NoFilePath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Visual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualType {
    Character,
    Line,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Command => write!(f, "COMMAND"),
            Mode::Visual => write!(f, "VISUAL"),
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
    pub command_buffer: String,
    pub visual_start: Option<Position>,
    pub visual_type: Option<VisualType>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            mode: Mode::Normal,
            cursor: Position { line: 1, col: 0 },
            file_path: None,
            dirty: false,
            command_buffer: String::new(),
            visual_start: None,
            visual_type: None,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PluginEvent {
    ModeChange { from: Mode, to: Mode },
    BufferChange,
    Key { mode: Mode, key: char },
    BufferSave { file_path: Option<PathBuf> },
}
