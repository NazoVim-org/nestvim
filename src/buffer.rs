use ropey::Rope;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct TextBuffer {
    doc: Rope,
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            doc: Rope::new(),
            file_path: None,
            dirty: false,
        }
    }

    pub fn with_text(text: &str) -> Self {
        Self {
            doc: Rope::from_str(text),
            file_path: None,
            dirty: false,
        }
    }

    pub fn line_count(&self) -> usize {
        self.doc.len_lines()
    }

    pub fn get_line(&self, line_number: usize) -> String {
        if line_number < 1 || line_number > self.line_count() {
            return String::new();
        }
        self.doc.line(line_number - 1).to_string()
    }

    pub fn insert(&mut self, line: usize, col: usize, text: &str) {
        let line_idx = line.saturating_sub(1);
        if line_idx >= self.doc.len_lines() {
            return;
        }
        
        let line_start = self.doc.line_to_char(line_idx);
        let char_idx = line_start + col.min(self.doc.line(line_idx).len_chars());
        
        self.doc.insert(char_idx, text);
        self.dirty = true;
    }

    pub fn delete(&mut self, line: usize, col: usize) {
        let line_idx = line.saturating_sub(1);
        if line_idx >= self.doc.len_lines() {
            return;
        }
        
        let line_start = self.doc.line_to_char(line_idx);
        let char_idx = line_start + col;
        
        if char_idx >= self.doc.len_chars() {
            return;
        }
        
        self.doc.remove(char_idx..char_idx + 1);
        self.dirty = true;
    }

    pub fn insert_char(&mut self, line: usize, col: usize, ch: char) {
        self.insert(line, col, &ch.to_string());
    }

    pub fn merge_with_prev_line(&mut self, line: usize) -> usize {
        if line <= 1 {
            return 0;
        }
        
        let prev_line_idx = line - 2;
        let cur_line_idx = line - 1;
        
        if prev_line_idx >= self.doc.len_lines() || cur_line_idx >= self.doc.len_lines() {
            return 0;
        }
        
        let prev_line_end = self.doc.line_to_char(prev_line_idx) + self.doc.line(prev_line_idx).len_chars();
        let cur_line_start = self.doc.line_to_char(cur_line_idx);
        
        // Remove the newline between lines
        if cur_line_start > 0 {
            self.doc.remove(cur_line_start - 1..cur_line_start);
            self.dirty = true;
        }
        
        prev_line_end.saturating_sub(1)
    }

    pub async fn load_file(path: &str) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut buffer = Self::with_text(&content);
        buffer.file_path = Some(PathBuf::from(path));
        Ok(buffer)
    }

    pub async fn save_file(&mut self) -> io::Result<()> {
        if let Some(path) = &self.file_path {
            let content = self.doc.to_string();
            fs::write(path, content)?;
            self.dirty = false;
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No file path set"))
        }
    }

    pub fn to_string(&self) -> String {
        self.doc.to_string()
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}
