use std::collections::HashMap;
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

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Clone, Debug, Default)]
pub struct Marks {
    marks: HashMap<char, Position>,
}

impl Marks {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set(&mut self, name: char, position: Position) {
        self.marks.insert(name, position);
    }
    
    pub fn get(&self, name: char) -> Option<Position> {
        self.marks.get(&name).copied()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Macros {
    macros: HashMap<char, Vec<String>>,
    recording: Option<char>,
}

impl Macros {
    pub fn new() -> Self {
        Self::default()
    }
    
pub fn start_recording(&mut self, name: char) {
        self.recording = Some(name);
        self.macros.entry(name).or_insert_with(Vec::new);
    }
    
    pub fn stop_recording(&mut self) -> Option<char> {
        self.recording.take()
    }
    
    pub fn add_key(&mut self, key: String) {
        if let Some(name) = self.recording {
            if let Some(keys) = self.macros.get_mut(&name) {
                keys.push(key);
            }
        }
    }
    
    pub fn get(&self, name: char) -> Option<&Vec<String>> {
        self.macros.get(&name)
    }
    
    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    pub marks: Marks,
    pub macros: Macros,
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
            marks: Marks::new(),
            macros: Macros::new(),
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
