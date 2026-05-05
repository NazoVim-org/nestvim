use ropey::Rope;
use std::path::PathBuf;
use crate::types::NestvimError;

pub struct TextBuffer {
    doc: Rope,
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
    modification_count: usize,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            doc: Rope::new(),
            file_path: None,
            dirty: false,
            modification_count: 0,
        }
    }

    pub fn with_text(text: &str) -> Self {
        Self {
            doc: Rope::from_str(text),
            file_path: None,
            dirty: false,
            modification_count: 0,
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
        self.modification_count += 1;
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
        self.modification_count += 1;
    }

    pub fn insert_char(&mut self, line: usize, col: usize, ch: char) {
        self.insert(line, col, &ch.to_string());
    }

    pub fn merge_with_prev_line(&mut self, line: usize) -> usize {
        if line <= 1 || line > self.line_count() {
            return 0;
        }
        
        let prev_line_idx = line - 2; // 0-indexed previous line
        let cur_line_idx = line - 1; // 0-indexed current line
        
        if prev_line_idx >= self.doc.len_lines() || cur_line_idx >= self.doc.len_lines() {
            return 0;
        }
        
        // Find the newline at the end of the previous line
        let prev_line_start = self.doc.line_to_char(prev_line_idx);
        let prev_line_len = self.doc.line(prev_line_idx).len_chars();
        let newline_pos = prev_line_start + prev_line_len - 1;
        
        if newline_pos >= self.doc.len_chars() {
            return 0;
        }
        
        // Remove the newline between lines
        self.doc.remove(newline_pos..newline_pos + 1);
        self.dirty = true;
        self.modification_count += 1;
        
        // Return the new cursor column (end of merged line)
        prev_line_len - 1
    }

    pub async fn load_file(path: &str) -> Result<Self, NestvimError> {
        let content = tokio::fs::read_to_string(path).await?;
        let mut buffer = Self::with_text(&content);
        buffer.file_path = Some(PathBuf::from(path));
        Ok(buffer)
    }

    pub async fn save_file(&mut self) -> Result<(), NestvimError> {
        if let Some(path) = &self.file_path {
            let content = self.doc.to_string();
            tokio::fs::write(path, content).await?;
            self.dirty = false;
            Ok(())
        } else {
            Err(NestvimError::NoFilePath)
        }
    }

    pub fn to_string(&self) -> String {
        self.doc.to_string()
    }

    pub fn modification_count(&self) -> usize {
        self.modification_count
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_char() {
        let mut buf = TextBuffer::new();
        buf.insert_char(1,0, 'a');
        assert_eq!(buf.get_line(1), "a");
        assert!(buf.dirty);
        assert_eq!(buf.modification_count(), 1);
    }

    #[test]
    fn test_delete_char() {
        let mut buf = TextBuffer::with_text("ab\n");
        buf.delete(1, 0);
        assert_eq!(buf.get_line(1), "b\n");
        assert!(buf.dirty);
    }

    #[test]
    fn test_merge_with_prev_line() {
        let mut buf = TextBuffer::with_text("hello\nworld\n");
        let col = buf.merge_with_prev_line(2);
        // Check content is merged correctly
        assert_eq!(buf.to_string(), "helloworld\n");
        assert!(buf.dirty);
    }

    #[test]
    fn test_save_resets_dirty() {
        let mut buf = TextBuffer::with_text("test\n");
        buf.file_path = Some(std::path::PathBuf::from("/tmp/test_nestvim.txt"));
        // Use tokio runtime to run async save
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            buf.save_file().await.expect("Save failed");
        });
        assert!(!buf.dirty);
    }
}
