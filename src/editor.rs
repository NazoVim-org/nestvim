use crate::buffer::TextBuffer;
use crate::highlight::Highlighter;
use crate::plugin::PluginManager;
use crate::register::Register;
use crate::renderer::Renderer;
use crate::terminal::Terminal;
use crate::types::{EditorState, Mode, PluginEvent, VisualType};
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures::StreamExt;
use std::io;
use tokio::time::{interval, Duration};

pub struct Editor {
    terminal: Terminal,
    buffer: TextBuffer,
    highlighter: Highlighter,
    renderer: Renderer,
    plugin_manager: PluginManager,
    register: Register,
    state: EditorState,
    running: bool,
    last_highlight_mod_count: usize,
    needs_render: bool,
    pending_operator: Option<char>,
    pending_register: Option<char>,
}

impl Editor {
    pub async fn new(file_path: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut terminal = Terminal::new()?;
        
        terminal.enable_raw_mode()?;
        
        let buffer = if let Some(path) = file_path {
            match TextBuffer::load_file(path).await {
                Ok(mut buf) => {
                    buf.file_path = Some(std::path::PathBuf::from(path));
                    buf
                }
                Err(e) => {
                    eprintln!("Failed to load file: {}", e);
                    TextBuffer::new()
                }
            }
        } else {
            TextBuffer::new()
        };
        
        let highlighter = Highlighter::new();
        
        let mut plugin_manager = PluginManager::new();
        if let Err(e) = plugin_manager.load_all() {
            eprintln!("Plugin loading warning: {}", e);
        }
        
        let renderer = Renderer::new();
        
        let register = Register::new();
        
        let state = EditorState {
            mode: Mode::Normal,
            cursor: crate::types::Position { line: 1, col: 0 },
            file_path: buffer.file_path.clone(),
            dirty: false,
            command_buffer: String::new(),
            visual_start: None,
            visual_type: None,
        };
        
        Ok(Self {
            terminal,
            buffer,
            highlighter,
            renderer,
            plugin_manager,
            register,
            state,
            running: false,
            last_highlight_mod_count: 0,
            needs_render: true,
            pending_operator: None,
            pending_register: None,
        })
    }
    
    pub async fn run(&mut self) -> io::Result<()> {
        self.running = true;
        
        // Event loop
        let mut reader = EventStream::new();
        let mut tick_interval = interval(Duration::from_millis(100));
        
        while self.running {
            self.state.dirty = self.buffer.dirty;
            
            tokio::select! {
                Some(event) = reader.next() => {
                    match event {
                        Ok(Event::Key(key)) => {
                            if key.kind == KeyEventKind::Press {
                                self.handle_key(key.code).await;
                            }
                        }
                        Ok(Event::Resize(_, _)) => {
                            self.terminal.update_size();
                            self.needs_render = true;
                        }
                        _ => {}
                    }
                }
                _ = tick_interval.tick() => {
                    // Periodic update if needed
                }
            }
            
            // Update highlighter only when buffer content changed
            if self.buffer.modification_count() > self.last_highlight_mod_count {
                let _ = self.highlighter.update(&self.buffer.to_string(), self.state.file_path.as_deref());
                self.last_highlight_mod_count = self.buffer.modification_count();
            }
            
            // Render only when needed
            if self.needs_render {
                self.renderer.render(&self.terminal, &self.buffer, &self.state);
                self.needs_render = false;
            }
        }
        
        Ok(())
    }
    
    async fn handle_key(&mut self, key: KeyCode) {
        if let KeyCode::Char(c) = key {
            self.plugin_manager.emit(PluginEvent::Key { mode: self.state.mode, key: c });
        }
        
        match self.state.mode {
            Mode::Normal => self.handle_normal(key).await,
            Mode::Insert => self.handle_insert(key).await,
            Mode::Command => self.handle_command(key).await,
            Mode::Visual => self.handle_visual(key).await,
        }
    }
    
    async fn handle_normal(&mut self, key: KeyCode) {
        let line_count = self.buffer.line_count();
        
        if let Some(op) = self.pending_operator {
            self.handle_operator(op, key).await;
            return;
        }
        
        match key {
            KeyCode::Char('h') => {
                self.state.cursor.col = self.state.cursor.col.saturating_sub(1);
                self.needs_render = true;
            }
            KeyCode::Char('l') => {
                let line_len = self.buffer.get_line(self.state.cursor.line).len();
                self.state.cursor.col = (self.state.cursor.col + 1).min(line_len.saturating_sub(1));
                self.needs_render = true;
            }
            KeyCode::Char('j') => {
                self.state.cursor.line = (self.state.cursor.line + 1).min(line_count);
                let len = self.buffer.get_line(self.state.cursor.line).len();
                self.state.cursor.col = self.state.cursor.col.min(len.saturating_sub(1));
                self.needs_render = true;
            }
            KeyCode::Char('k') => {
                self.state.cursor.line = self.state.cursor.line.saturating_sub(1).max(1);
                let len = self.buffer.get_line(self.state.cursor.line).len();
                self.state.cursor.col = self.state.cursor.col.min(len.saturating_sub(1));
                self.needs_render = true;
            }
            KeyCode::Char('i') => {
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Insert;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Insert });
                self.needs_render = true;
            }
            KeyCode::Char(':') => {
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Command;
                self.state.command_buffer.clear();
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Command });
                self.needs_render = true;
            }
            KeyCode::Char('v') => {
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Visual;
                self.state.visual_start = Some(self.state.cursor.clone());
                self.state.visual_type = Some(VisualType::Character);
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Visual });
                self.needs_render = true;
            }
            KeyCode::Char('V') => {
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Visual;
                self.state.visual_start = Some(self.state.cursor.clone());
                self.state.visual_type = Some(VisualType::Line);
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Visual });
                self.needs_render = true;
            }
            KeyCode::Char('y') => {
                self.pending_operator = Some('y');
            }
            KeyCode::Char('d') => {
                self.pending_operator = Some('d');
            }
            KeyCode::Char('p') => {
                self.paste(false).await;
            }
            KeyCode::Char('P') => {
                self.paste(true).await;
            }
            KeyCode::Char('"') => {
                self.pending_register = Some('"');
            }
            _ => {
                if let Some(_r) = self.pending_register {
                    if let KeyCode::Char(c) = key {
                        if c >= 'a' && c <= 'z' {
                            self.pending_register = Some(c);
                            return;
                        }
                    }
                    self.pending_register = None;
                }
            }
        }
        
        if let Some(r) = self.pending_register {
            if let KeyCode::Char(c) = key {
                if c >= 'a' && c <= 'z' {
                    self.pending_register = None;
                    if let Some(op) = self.pending_operator {
                        self.pending_operator = None;
                        let reg = r;
                        self.execute_operator_with_register(op, reg, key).await;
                        return;
                    }
                }
            }
        }
    }
    
    async fn handle_operator(&mut self, op: char, key: KeyCode) {
        let register = self.pending_register.unwrap_or('"');
        self.pending_operator = None;
        self.pending_register = None;
        
        match op {
            'y' => {
                match key {
                    KeyCode::Char('y') => {
                        self.yank_line(register);
                    }
                    KeyCode::Char('w') => {
                        self.yank_word(register);
                    }
                    KeyCode::Char('e') => {
                        self.yank_word_end(register);
                    }
                    _ => {}
                }
            }
            'd' => {
                match key {
                    KeyCode::Char('d') => {
                        self.delete_line(register);
                    }
                    KeyCode::Char('w') => {
                        self.delete_word(register);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    
    async fn execute_operator_with_register(&mut self, op: char, register: char, _key: KeyCode) {
        match op {
            'y' => {
                self.yank_line(register);
            }
            'd' => {
                self.delete_line(register);
            }
            _ => {}
        }
    }
    
    fn yank_line(&mut self, register: char) {
        let line = self.state.cursor.line;
        let content = self.buffer.get_line_range(line, line);
        self.register.set(register, &content);
        self.needs_render = true;
    }
    
    fn yank_word(&mut self, register: char) {
        let (word, _, _) = self.buffer.get_word_range(self.state.cursor.line, self.state.cursor.col);
        self.register.set(register, &word);
        self.needs_render = true;
    }
    
    fn yank_word_end(&mut self, register: char) {
        let line = self.state.cursor.line;
        let col = self.state.cursor.col;
        let line_str = self.buffer.get_line(line);
        let chars: Vec<char> = line_str.chars().collect();
        
        let mut end = col;
        while end < chars.len() && !chars[end].is_whitespace() {
            end += 1;
        }
        
        let word: String = chars[col..end].iter().collect();
        self.register.set(register, &word);
        self.needs_render = true;
    }
    
    fn delete_line(&mut self, register: char) {
        let line = self.state.cursor.line;
        let content = self.buffer.delete_line(line);
        self.register.set(register, &content);
        
        let line_count = self.buffer.line_count();
        if line > line_count {
            self.state.cursor.line = line_count.max(1);
        }
        let len = self.buffer.get_line(self.state.cursor.line).len();
        self.state.cursor.col = self.state.cursor.col.min(len.saturating_sub(1));
        self.needs_render = true;
    }
    
    fn delete_word(&mut self, register: char) {
        let (word, start, _) = self.buffer.get_word_range(self.state.cursor.line, self.state.cursor.col);
        if !word.is_empty() {
            let char_start = self.buffer.line_to_char(self.state.cursor.line - 1) + start;
            let char_end = char_start + word.len();
            self.buffer.remove_range(char_start, char_end);
            self.register.set(register, &word);
        }
        self.needs_render = true;
    }
    
    async fn paste(&mut self, before: bool) {
        let content = self.register.get_default();
        if content.is_empty() {
            return;
        }
        
        if content.contains('\n') {
            let lines: Vec<&str> = content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if i == 0 {
                    if before {
                        self.buffer.insert(self.state.cursor.line, 0, line);
                        self.buffer.insert(self.state.cursor.line, line.len(), "\n");
                    } else {
                        self.buffer.insert(self.state.cursor.line, self.state.cursor.col, line);
                        self.buffer.insert(self.state.cursor.line, self.state.cursor.col + line.len(), "\n");
                    }
                } else {
                    let insert_line = if before { self.state.cursor.line + i } else { self.state.cursor.line + i };
                    self.buffer.insert(insert_line, 0, line);
                    self.buffer.insert(insert_line, line.len(), "\n");
                }
            }
            if before {
                self.state.cursor.line += lines.len() - 1;
                self.state.cursor.col = lines.last().map(|l| l.len()).unwrap_or(0);
            } else {
                self.state.cursor.line += lines.len() - 1;
                let last_line = lines.last().unwrap();
                self.state.cursor.col = last_line.len();
            }
        } else {
            if before {
                self.buffer.insert(self.state.cursor.line, self.state.cursor.col, &content);
            } else {
                self.buffer.insert(self.state.cursor.line, self.state.cursor.col + 1, &content);
                self.state.cursor.col += 1;
            }
            if !before {
                self.state.cursor.col += content.len();
            }
        }
        
        self.on_buffer_modified();
    }
    
    async fn handle_visual(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Normal;
                self.state.visual_start = None;
                self.state.visual_type = None;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Normal });
                self.needs_render = true;
            }
            KeyCode::Char('y') => {
                self.visual_yank();
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Normal;
                self.state.visual_start = None;
                self.state.visual_type = None;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Normal });
                self.needs_render = true;
            }
            KeyCode::Char('d') => {
                self.visual_delete();
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Normal;
                self.state.visual_start = None;
                self.state.visual_type = None;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Normal });
                self.needs_render = true;
            }
            KeyCode::Char('h') => {
                self.state.cursor.col = self.state.cursor.col.saturating_sub(1);
                self.needs_render = true;
            }
            KeyCode::Char('l') => {
                let line_len = self.buffer.get_line(self.state.cursor.line).len();
                self.state.cursor.col = (self.state.cursor.col + 1).min(line_len.saturating_sub(1));
                self.needs_render = true;
            }
            KeyCode::Char('j') => {
                let line_count = self.buffer.line_count();
                self.state.cursor.line = (self.state.cursor.line + 1).min(line_count);
                let len = self.buffer.get_line(self.state.cursor.line).len();
                self.state.cursor.col = self.state.cursor.col.min(len.saturating_sub(1));
                self.needs_render = true;
            }
            KeyCode::Char('k') => {
                self.state.cursor.line = self.state.cursor.line.saturating_sub(1).max(1);
                let len = self.buffer.get_line(self.state.cursor.line).len();
                self.state.cursor.col = self.state.cursor.col.min(len.saturating_sub(1));
                self.needs_render = true;
            }
            _ => {}
        }
    }
    
    fn visual_yank(&mut self) {
        if let (Some(start), Some(vtype)) = (&self.state.visual_start, self.state.visual_type) {
            let content = match vtype {
                VisualType::Character => {
                    let (s_line, s_col, e_line, e_col) = self.normalize_selection(start, &self.state.cursor);
                    self.buffer.get_char_range(s_line, s_col, e_line, e_col)
                }
                VisualType::Line => {
                    let (s_line, e_line) = self.normalize_line_selection(start.line, self.state.cursor.line);
                    self.buffer.get_line_range(s_line, e_line)
                }
            };
            self.register.set('"', &content);
        }
    }
    
    fn visual_delete(&mut self) {
        if let (Some(start), Some(vtype)) = (&self.state.visual_start, self.state.visual_type) {
            let content = match vtype {
                VisualType::Character => {
                    let (s_line, s_col, e_line, e_col) = self.normalize_selection(start, &self.state.cursor);
                    self.buffer.delete_range(s_line, s_col, e_line, e_col)
                }
                VisualType::Line => {
                    let (s_line, e_line) = self.normalize_line_selection(start.line, self.state.cursor.line);
                    let mut content = String::new();
                    for line in s_line..=e_line {
                        content.push_str(&self.buffer.delete_line(line));
                        if line < e_line {
                            content.push('\n');
                        }
                    }
                    content
                }
            };
            self.register.set('"', &content);
            self.on_buffer_modified();
        }
    }
    
    fn normalize_selection(&self, start: &crate::types::Position, end: &crate::types::Position) -> (usize, usize, usize, usize) {
        if start.line < end.line || (start.line == end.line && start.col <= end.col) {
            (start.line, start.col, end.line, end.col + 1)
        } else {
            (end.line, end.col, start.line, start.col + 1)
        }
    }
    
    fn normalize_line_selection(&self, start_line: usize, end_line: usize) -> (usize, usize) {
        if start_line <= end_line {
            (start_line, end_line)
        } else {
            (end_line, start_line)
        }
    }
    
    #[allow(dead_code)]
    fn line_to_char(&self, line_idx: usize) -> usize {
        self.buffer.line_to_char(line_idx)
    }
    
    async fn handle_insert(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Normal;
                self.state.cursor.col = self.state.cursor.col.saturating_sub(1);
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Normal });
                self.needs_render = true;
            }
            KeyCode::Backspace => {
                if self.state.cursor.col > 0 {
                    self.buffer.delete(self.state.cursor.line, self.state.cursor.col - 1);
                    self.state.cursor.col -= 1;
                    self.on_buffer_modified();
                } else if self.state.cursor.line > 1 {
                    let new_col = self.buffer.merge_with_prev_line(self.state.cursor.line);
                    self.state.cursor.line -= 1;
                    self.state.cursor.col = new_col;
                    self.on_buffer_modified();
                }
            }
            KeyCode::Enter => {
                self.buffer.insert(self.state.cursor.line, self.state.cursor.col, "\n");
                self.state.cursor.line += 1;
                self.state.cursor.col = 0;
                self.on_buffer_modified();
            }
            KeyCode::Char(c) => {
                self.buffer.insert_char(self.state.cursor.line, self.state.cursor.col, c);
                self.state.cursor.col += 1;
                self.on_buffer_modified();
            }
            _ => {}
        }
    }
    
    fn on_buffer_modified(&mut self) {
        self.plugin_manager.emit(PluginEvent::BufferChange);
        self.needs_render = true;
    }
    
    async fn handle_command(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                let cmd = self.state.command_buffer.trim().to_string();
                self.state.command_buffer.clear();
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Normal;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Normal });
                self.needs_render = true;
                
                match cmd.as_str() {
                    "q" => {
                        self.running = false;
                    }
                    "w" => {
                        if let Err(e) = self.buffer.save_file().await {
                            eprintln!("[editor] Save failed: {}", e);
                        } else {
                            self.plugin_manager.emit(PluginEvent::BufferSave { file_path: self.state.file_path.clone() });
                        }
                    }
                    "wq" => {
                        if let Err(e) = self.buffer.save_file().await {
                            eprintln!("[editor] Save failed: {}", e);
                        } else {
                            self.plugin_manager.emit(PluginEvent::BufferSave { file_path: self.state.file_path.clone() });
                            self.running = false;
                        }
                    }
                    _ => {
                        if !self.plugin_manager.execute_command(&cmd) {
                            eprintln!("[editor] Unknown command: {}", cmd);
                        }
                    }
                }
            }
            KeyCode::Esc => {
                self.state.command_buffer.clear();
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Normal;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Normal });
                self.needs_render = true;
            }
            KeyCode::Backspace => {
                self.state.command_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.state.command_buffer.push(c);
            }
            _ => {}
        }
    }
}
