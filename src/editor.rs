use crate::buffer::TextBuffer;
use crate::highlight::Highlighter;
use crate::plugin::PluginManager;
use crate::register::Register;
use crate::renderer::Renderer;
use crate::terminal::Terminal;
use crate::types::{ConfirmAction, EditorState, Mode, PluginEvent, SearchDirection, SearchResult, VisualType};
use crate::undo::UndoManager;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use std::io;
use std::time::{Duration, Instant};

pub struct Editor {
    terminal: Terminal,
    buffer: TextBuffer,
    highlighter: Highlighter,
    renderer: Renderer,
    plugin_manager: PluginManager,
    register: Register,
    undo_manager: UndoManager,
    state: EditorState,
    running: bool,
    last_highlight_mod_count: usize,
    last_keypress_time: Instant,
    needs_render: bool,
    pending_operator: Option<char>,
    pending_register: Option<char>,
    pending_mark: Option<char>,
    pending_macro_play: Option<char>,
    search_query: String,
    search_direction: SearchDirection,
    search_results: Vec<SearchResult>,
    current_search_idx: usize,
    dot_last_action: Option<DotAction>,
    replace_char: Option<char>,
}

#[derive(Clone)]
enum DotAction {
    Insert { text: String },
    Delete { text: String, line: usize, col: usize },
    Change { text: String, line: usize, col: usize },
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
        
        let undo_manager = UndoManager::new();
        
        let state = EditorState {
            mode: Mode::Normal,
            cursor: crate::types::Position { line: 1, col: 0 },
            file_path: buffer.file_path.clone(),
            dirty: false,
            command_buffer: String::new(),
            visual_start: None,
            visual_type: None,
            marks: crate::types::Marks::new(),
            macros: crate::types::Macros::new(),
            confirmation_prompt: None,
            show_line_numbers: true,
        };
        
        Ok(Self {
            terminal,
            buffer,
            highlighter,
            renderer,
            plugin_manager,
            register,
            undo_manager,
            state,
            running: false,
            last_highlight_mod_count: 0,
            last_keypress_time: Instant::now(),
            needs_render: true,
            pending_operator: None,
            pending_register: None,
            pending_mark: None,
            pending_macro_play: None,
            search_query: String::new(),
            search_direction: SearchDirection::Forward,
            search_results: Vec::new(),
            current_search_idx: 0,
            dot_last_action: None,
            replace_char: None,
        })
    }
    
    pub async fn run(&mut self) -> io::Result<()> {
        self.running = true;

        if let Err(e) = self.renderer.render(&mut self.terminal, &self.buffer, &self.state) {
            eprintln!("[render] error: {}", e);
        }
        self.needs_render = false;
        
        let mut reader = EventStream::new();
        
        while self.running {
            self.state.dirty = self.buffer.dirty;
            
            tokio::select! {
                Some(event) = reader.next() => {
                    match event {
                        Ok(Event::Key(key)) => {
                            if key.kind == KeyEventKind::Press {
                                self.last_keypress_time = Instant::now();
                                let modifiers = key.modifiers;
                                self.handle_key(key.code, modifiers).await;
                            }
                        }
                        Ok(Event::Resize(_, _)) => {
                            self.terminal.update_size();
                            self.terminal.clear_cache();
                            self.needs_render = true;
                        }
                        _ => {}
                    }
                }
            }
            
            let now = Instant::now();
            if self.buffer.modification_count() > self.last_highlight_mod_count
                && now.duration_since(self.last_keypress_time) > Duration::from_millis(150)
            {
                let _ = self.highlighter.update(&self.buffer.to_string(), self.state.file_path.as_deref());
                self.last_highlight_mod_count = self.buffer.modification_count();
            }
            
            if self.needs_render {
                if let Err(e) = self.renderer.render(&mut self.terminal, &self.buffer, &self.state) {
                    eprintln!("[render] error: {}", e);
                }
                self.needs_render = false;
            }
        }
        
        Ok(())
    }
    
    async fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        if let KeyCode::Char(c) = key {
            self.plugin_manager.emit(PluginEvent::Key { mode: self.state.mode, key: c });
        }
        
        if modifiers.contains(KeyModifiers::CONTROL) {
            match self.state.mode {
                Mode::Normal | Mode::Insert | Mode::Replace => {
                    match key {
                        KeyCode::Char('d') => {
                            self.scroll_by(self.terminal.rows() as usize / 2);
                            self.needs_render = true;
                            return;
                        }
                        KeyCode::Char('u') => {
                            self.scroll_by(self.terminal.rows() as usize / 2);
                            self.needs_render = true;
                            return;
                        }
                        KeyCode::Char('y') => {
                            self.scroll_up_one();
                            self.needs_render = true;
                            return;
                        }
                        KeyCode::Char('e') => {
                            self.scroll_down_one();
                            self.needs_render = true;
                            return;
                        }
                        KeyCode::Char('g') => {
                            self.show_file_info();
                            self.needs_render = true;
                            return;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        match self.state.mode {
            Mode::Normal => self.handle_normal(key).await,
            Mode::Insert => self.handle_insert(key).await,
            Mode::Command => self.handle_command(key).await,
            Mode::Visual => self.handle_visual(key).await,
            Mode::Replace => self.handle_replace(key).await,
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
            KeyCode::Char('/') => {
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Command;
                self.state.command_buffer.clear();
                self.state.command_buffer.push('/');
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Command });
                self.needs_render = true;
            }
            KeyCode::Char('*') => {
                self.search_word_under_cursor();
                self.needs_render = true;
            }
            KeyCode::Char('n') => {
                self.search_next();
                self.needs_render = true;
            }
            KeyCode::Char('N') => {
                self.search_prev();
                self.needs_render = true;
            }
            KeyCode::Char('u') => {
                self.undo();
            }
            KeyCode::Char('r') => {
                if self.state.command_buffer.is_empty() {
                    let prev_mode = self.state.mode;
                    self.state.mode = Mode::Replace;
                    self.replace_char = None;
                    self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Replace });
                    self.needs_render = true;
                }
            }
            KeyCode::Char('R') => {
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Replace;
                self.replace_char = None;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Replace });
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
            KeyCode::Char('m') => {
                self.pending_mark = Some('m');
            }
            KeyCode::Char('`') => {
                self.pending_mark = Some('`');
            }
            KeyCode::Char('\'') => {
                self.pending_mark = Some('\'');
            }
            KeyCode::Char('q') => {
                if let KeyCode::Char(c) = key {
                    if c >= 'a' && c <= 'z' {
                        self.toggle_macro_recording(c);
                        self.needs_render = true;
                        return;
                    }
                }
            }
            KeyCode::Char('@') => {
                self.pending_macro_play = Some('@');
            }
            _ => {
                match key {
                    KeyCode::Char('g') => {
                        self.pending_operator = Some('g');
                    }
                    KeyCode::Char('G') => {
                        self.state.cursor.line = self.buffer.line_count().max(1);
                        let len = self.buffer.get_line(self.state.cursor.line).len();
                        self.state.cursor.col = len.saturating_sub(1);
                        self.needs_render = true;
                    }
                    KeyCode::Char('%') => {
                        self.jump_to_matching_bracket();
                        self.needs_render = true;
                    }
                    KeyCode::Char('x') => {
                        self.delete_char('"');
                        self.needs_render = true;
                    }
                    KeyCode::Char('.') => {
                        self.repeat_last_action().await;
                        self.needs_render = true;
                    }
                    KeyCode::Char('w') => {
                        self.move_word_forward();
                        self.needs_render = true;
                    }
                    KeyCode::Char('b') => {
                        self.move_word_backward();
                        self.needs_render = true;
                    }
                    KeyCode::Char('e') => {
                        self.move_word_end();
                        self.needs_render = true;
                    }
                    KeyCode::Char('o') => {
                        self.open_line(false);
                        self.needs_render = true;
                    }
                    KeyCode::Char('O') => {
                        self.open_line(true);
                        self.needs_render = true;
                    }
                    KeyCode::Char('a') => {
                        let prev_mode = self.state.mode;
                        self.state.mode = Mode::Insert;
                        self.state.cursor.col = (self.state.cursor.col + 1).min(self.buffer.get_line(self.state.cursor.line).len().saturating_sub(1));
                        self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Insert });
                        self.needs_render = true;
                    }
                    KeyCode::Char('A') => {
                        let prev_mode = self.state.mode;
                        self.state.mode = Mode::Insert;
                        let line_len = self.buffer.get_line(self.state.cursor.line).len();
                        self.state.cursor.col = line_len.saturating_sub(1);
                        self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Insert });
                        self.needs_render = true;
                    }
                    KeyCode::Char('c') => {
                        self.pending_operator = Some('c');
                    }
                    KeyCode::Char('J') => {
                        self.join_lines();
                        self.needs_render = true;
                    }
                    KeyCode::Char('>') => {
                        self.pending_operator = Some('>');
                    }
                    KeyCode::Char('<') => {
                        self.pending_operator = Some('<');
                    }
                    KeyCode::Char('z') => {
                        self.pending_operator = Some('z');
                    }
                    KeyCode::Char('s') => {
                        self.delete_char('"');
                        let prev_mode = self.state.mode;
                        self.state.mode = Mode::Insert;
                        self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Insert });
                        self.needs_render = true;
                    }
                    _ => {}
                }

                if let Some(_pending) = self.pending_macro_play {
                    if let KeyCode::Char(c) = key {
                        if c >= 'a' && c <= 'z' {
                            self.play_macro(c);
                            self.pending_macro_play = None;
                            self.needs_render = true;
                            return;
                        }
                    }
                    self.pending_macro_play = None;
                }
                if let Some(op) = self.pending_operator {
                    if op == 'g' {
                        if let KeyCode::Char('g') = key {
                            self.state.cursor.line = 1;
                            self.state.cursor.col = 0;
                        }
                        self.pending_operator = None;
                        self.needs_render = true;
                        return;
                    }
                    if let KeyCode::Char('z') = key {
                        match key {
                            KeyCode::Char('z') => {
                                self.scroll_cursor_to_center();
                            }
                            KeyCode::Char('t') => {
                                self.scroll_cursor_to_top();
                            }
                            KeyCode::Char('b') => {
                                self.scroll_cursor_to_bottom();
                            }
                            _ => {}
                        }
                        self.pending_operator = None;
                        self.needs_render = true;
                        return;
                    }
                }
                if let Some(pending) = self.pending_mark {
                    if let KeyCode::Char(c) = key {
                        if (pending == 'm' && c >= 'a' && c <= 'z') || (pending == '`' || pending == '\'') {
                            self.handle_mark(pending, c);
                            self.pending_mark = None;
                            self.needs_render = true;
                            return;
                        }
                    }
                    self.pending_mark = None;
                }
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
                    KeyCode::Char('i') => {
                        self.handle_text_object(key, register, true).await;
                    }
                    KeyCode::Char('a') => {
                        self.handle_text_object(key, register, false).await;
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
                    KeyCode::Char('i') => {
                        self.handle_text_object(key, register, true).await;
                    }
                    KeyCode::Char('a') => {
                        self.handle_text_object(key, register, false).await;
                    }
                    _ => {}
                }
            }
            'c' => {
                match key {
                    KeyCode::Char('c') => {
                        let content = self.buffer.get_line(self.state.cursor.line);
                        self.register.set(register, &content);
                        self.buffer.delete_line(self.state.cursor.line);
                        let prev_mode = self.state.mode;
                        self.state.mode = Mode::Insert;
                        self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Insert });
                        self.dot_last_action = Some(DotAction::Change { text: content, line: self.state.cursor.line, col: 0 });
                        self.on_buffer_modified();
                    }
                    KeyCode::Char('w') => {
                        let (word, start, _) = self.buffer.get_word_range(self.state.cursor.line, self.state.cursor.col);
                        if !word.is_empty() {
                            let char_start = self.buffer.line_to_char(self.state.cursor.line - 1) + start;
                            let char_end = char_start + word.len();
                            let content = self.buffer.get_char_range(self.state.cursor.line, start, self.state.cursor.line, start + word.len());
                            self.register.set(register, &content);
                            self.buffer.remove_range(char_start, char_end);
                            let prev_mode = self.state.mode;
                            self.state.mode = Mode::Insert;
                            self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Insert });
                            self.dot_last_action = Some(DotAction::Change { text: content, line: self.state.cursor.line, col: start });
                            self.on_buffer_modified();
                        }
                    }
                    KeyCode::Char('i') => {
                        self.handle_text_object_change(key, register, true).await;
                    }
                    KeyCode::Char('a') => {
                        self.handle_text_object_change(key, register, false).await;
                    }
                    _ => {}
                }
            }
            '>' => {
                match key {
                    KeyCode::Char('>') => {
                        self.indent_lines(register, true);
                    }
                    _ => {}
                }
            }
            '<' => {
                match key {
                    KeyCode::Char('<') => {
                        self.indent_lines(register, false);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    async fn handle_text_object_change(&mut self, key: KeyCode, register: char, inner: bool) {
        match key {
            KeyCode::Char('w') => {
                if inner {
                    let (word, start, _) = self.buffer.get_word_range(self.state.cursor.line, self.state.cursor.col);
                    if !word.is_empty() {
                        let content = self.buffer.get_char_range(self.state.cursor.line, start, self.state.cursor.line, start + word.len());
                        self.register.set(register, &content);
                        let char_start = self.buffer.line_to_char(self.state.cursor.line - 1) + start;
                        let char_end = char_start + word.len();
                        self.buffer.remove_range(char_start, char_end);
                    }
                }
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Insert;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Insert });
                self.on_buffer_modified();
            }
            _ => {}
        }
    }

    fn indent_lines(&mut self, register: char, indent: bool) {
        let line = self.state.cursor.line;
        let content = self.buffer.get_line(line);
        let _ = register;
        
        if indent {
            self.buffer.insert(line, 0, "\t");
        } else if content.starts_with('\t') {
            self.buffer.delete(line, 0);
        } else if content.starts_with("  ") {
            self.buffer.delete(line, 0);
            self.buffer.delete(line, 0);
        }
        
        self.needs_render = true;
    }
    
    async fn handle_text_object(&mut self, key: KeyCode, register: char, inner: bool) {
        match key {
            KeyCode::Char('w') => {
                if inner {
                    let (word, start, _) = self.buffer.get_word_range(self.state.cursor.line, self.state.cursor.col);
                    if !word.is_empty() {
                        let char_start = self.buffer.line_to_char(self.state.cursor.line - 1) + start;
                        let char_end = char_start + word.len();
                        let content = self.buffer.get_char_range(self.state.cursor.line, start, self.state.cursor.line, start + word.len());
                        self.register.set(register, &content);
                        self.buffer.remove_range(char_start, char_end);
                    }
                } else {
                    let (word, start, end) = self.buffer.get_word_range(self.state.cursor.line, self.state.cursor.col);
                    if !word.is_empty() {
                        let mut search_start = start;
                        while search_start > 0 {
                            search_start -= 1;
                        }
                        let mut search_end = end;
                        while search_end < self.buffer.get_line(self.state.cursor.line).len() {
                            search_end += 1;
                        }
                    }
                }
            }
            _ => {}
        }
        self.needs_render = true;
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
    
    fn undo(&mut self) {
        if let Some(edit) = self.undo_manager.undo(&mut self.buffer) {
            self.state.cursor = edit.cursor_before;
            self.needs_render = true;
        }
    }
    
    #[allow(dead_code)]
    fn redo(&mut self) {
        if let Some(edit) = self.undo_manager.redo(&mut self.buffer) {
            self.state.cursor = edit.cursor_after;
            self.needs_render = true;
        }
    }
    
    fn search_word_under_cursor(&mut self) {
        let (word, _, _) = self.buffer.get_word_range(self.state.cursor.line, self.state.cursor.col);
        if !word.is_empty() {
            let query = word.clone();
            self.do_search(&query, SearchDirection::Forward);
        }
    }
    
    fn search_next(&mut self) {
        if !self.search_query.is_empty() {
            let query = self.search_query.clone();
            let dir = self.search_direction;
            self.do_search(&query, dir);
        }
    }
    
    fn search_prev(&mut self) {
        if !self.search_query.is_empty() {
            let query = self.search_query.clone();
            let dir = match self.search_direction {
                SearchDirection::Forward => SearchDirection::Backward,
                SearchDirection::Backward => SearchDirection::Forward,
            };
            self.do_search(&query, dir);
        }
    }
    
    fn do_search(&mut self, query: &str, direction: SearchDirection) {
        self.search_query = query.to_string();
        self.search_direction = direction;
        self.search_results = self.buffer.search(query);
        
        if self.search_results.is_empty() {
            return;
        }
        
        if direction == SearchDirection::Forward {
            self.current_search_idx = self.search_results.iter()
                .position(|r| r.line > self.state.cursor.line || (r.line == self.state.cursor.line && r.start_col > self.state.cursor.col))
                .unwrap_or(0);
        } else {
            self.current_search_idx = self.search_results.iter()
                .rposition(|r| r.line < self.state.cursor.line || (r.line == self.state.cursor.line && r.start_col < self.state.cursor.col))
                .unwrap_or(self.search_results.len() - 1);
        }
        
        if let Some(result) = self.search_results.get(self.current_search_idx) {
            self.state.cursor.line = result.line;
            self.state.cursor.col = result.start_col;
        }
    }
    
    fn handle_mark(&mut self, action: char, name: char) {
        if action == 'm' {
            self.state.marks.set(name, self.state.cursor);
        } else if let Some(pos) = self.state.marks.get(name) {
            self.state.cursor = pos;
        }
    }
    
    fn toggle_macro_recording(&mut self, name: char) {
        if self.state.macros.is_recording() {
            self.state.macros.stop_recording();
        } else {
            self.state.macros.start_recording(name);
        }
    }
    
    fn play_macro(&mut self, name: char) {
        let keys = self.state.macros.get(name).cloned();
        if let Some(keys) = keys {
            for key_str in keys {
                self.execute_macro_key(&key_str);
            }
        }
    }
    
    fn execute_macro_key(&mut self, key_str: &str) {
        use crossterm::event::KeyCode;
        let key = match key_str {
            "h" => KeyCode::Char('h'),
            "j" => KeyCode::Char('j'),
            "k" => KeyCode::Char('k'),
            "l" => KeyCode::Char('l'),
            "i" => KeyCode::Char('i'),
            "a" => KeyCode::Char('a'),
            "x" => KeyCode::Char('x'),
            "dd" => KeyCode::Char('d'),
            "yy" => KeyCode::Char('y'),
            _ => return,
        };
        match key {
            KeyCode::Char('j') => {
                self.state.cursor.line = (self.state.cursor.line + 1).min(self.buffer.line_count());
            }
            KeyCode::Char('k') => {
                self.state.cursor.line = self.state.cursor.line.saturating_sub(1).max(1);
            }
            KeyCode::Char('h') => {
                self.state.cursor.col = self.state.cursor.col.saturating_sub(1);
            }
            KeyCode::Char('l') => {
                let line_len = self.buffer.get_line(self.state.cursor.line).len();
                self.state.cursor.col = (self.state.cursor.col + 1).min(line_len.saturating_sub(1));
            }
            _ => {}
        }
        self.needs_render = true;
    }
    
    async fn handle_command(&mut self, key: KeyCode) {
        if self.state.has_confirmation() {
            self.handle_confirmation(key).await;
            return;
        }

        match key {
            KeyCode::Enter => {
                let cmd = self.state.command_buffer.trim().to_string();
                self.state.command_buffer.clear();
                
                match cmd.as_str() {
                    "q" => {
                        self.handle_quit().await;
                    }
                    "q!" => {
                        self.running = false;
                    }
                    "w" => {
                        if let Err(e) = self.buffer.save_file().await {
                            eprintln!("[editor] Save failed: {}", e);
                        } else {
                            self.plugin_manager.emit(PluginEvent::BufferSave { file_path: self.state.file_path.clone() });
                        }
                    }
                    "w!" => {
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
                    "wq!" => {
                        if let Err(e) = self.buffer.save_file().await {
                            eprintln!("[editor] Save failed: {}", e);
                        } else {
                            self.plugin_manager.emit(PluginEvent::BufferSave { file_path: self.state.file_path.clone() });
                            self.running = false;
                        }
                    }
                    "wqa" | "wa" => {
                        if let Err(e) = self.buffer.save_file().await {
                            eprintln!("[editor] Save failed: {}", e);
                        } else {
                            self.plugin_manager.emit(PluginEvent::BufferSave { file_path: self.state.file_path.clone() });
                            self.running = false;
                        }
                    }
                    "qa" => {
                        self.running = false;
                    }
                    "e" => {
                        self.reload_file().await;
                    }
                    "e!" => {
                        self.reload_file_discard().await;
                    }
                    _ => {
                        if cmd.starts_with("set ") {
                            self.handle_set_command(&cmd);
                        } else if !self.plugin_manager.execute_command(&cmd) {
                            eprintln!("[editor] Unknown command: {}", cmd);
                        }
                    }
                }

                if !self.state.has_confirmation() {
                    let prev_mode = self.state.mode;
                    self.state.mode = Mode::Normal;
                    self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Normal });
                    self.needs_render = true;
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
                self.needs_render = true;
            }
            KeyCode::Char(c) => {
                self.state.command_buffer.push(c);
                self.needs_render = true;
            }
            _ => {}
        }
    }
    
    async fn handle_quit(&mut self) {
        if self.buffer.dirty {
            self.state.set_confirmation(
                "No write since last change. Quit anyway? (y/n Enter/Esc: yes, n: no)".to_string(),
                ConfirmAction::Quit,
            );
            self.needs_render = true;
        } else {
            self.running = false;
        }
    }
    
    async fn handle_confirmation(&mut self, key: KeyCode) {
        let should_quit = match key {
            KeyCode::Char('y') | KeyCode::Enter => true,
            KeyCode::Char('n') | KeyCode::Esc => false,
            _ => {
                self.state.clear_confirmation();
                self.needs_render = true;
                return;
            }
        };
        
        let action = self.state.confirmation_prompt.as_ref().unwrap().action.clone();
        self.state.clear_confirmation();
        self.needs_render = true;
        
        match action {
            ConfirmAction::Quit => {
                if should_quit {
                    self.running = false;
                }
            }
            ConfirmAction::QuitDiscard => {
                self.running = false;
            }
            ConfirmAction::WriteQuitAll => {
                if !should_quit {
                    return;
                }
                if let Err(e) = self.buffer.save_file().await {
                    eprintln!("[editor] Save failed: {}", e);
                } else {
                    self.plugin_manager.emit(PluginEvent::BufferSave { file_path: self.state.file_path.clone() });
                    self.running = false;
                }
            }
        }
    }

    async fn reload_file(&mut self) {
        if let Some(path) = &self.state.file_path {
            match TextBuffer::load_file(path.to_str().unwrap_or("")).await {
                Ok(buf) => {
                    self.buffer = buf;
                }
                Err(e) => {
                    eprintln!("[editor] Failed to reload file: {}", e);
                }
            }
        }
    }

    async fn reload_file_discard(&mut self) {
        self.buffer.dirty = false;
        self.reload_file().await;
    }

    fn handle_set_command(&mut self, cmd: &str) {
        let args = cmd.trim_start_matches("set ").trim();
        
        match args {
            "number" => {
                self.state.show_line_numbers = true;
            }
            "nonumber" | "nonumber!" => {
                self.state.show_line_numbers = false;
            }
            "number!" => {
                self.state.show_line_numbers = !self.state.show_line_numbers;
            }
            _ => {
                eprintln!("[editor] Unknown set option: {}", args);
            }
        }
    }

    async fn handle_replace(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Normal;
                self.replace_char = None;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Normal });
                self.needs_render = true;
            }
            KeyCode::Char(c) => {
                let line = self.state.cursor.line;
                let col = self.state.cursor.col;
                if col < self.buffer.get_line(line).len() {
                    self.buffer.delete(line, col);
                }
                self.buffer.insert_char(line, col, c);
                self.state.cursor.col += 1;
                self.on_buffer_modified();
            }
            KeyCode::Backspace => {
                if self.state.cursor.col > 0 {
                    self.state.cursor.col -= 1;
                }
                self.needs_render = true;
            }
            KeyCode::Enter => {
                self.buffer.insert(self.state.cursor.line, self.state.cursor.col, "\n");
                self.state.cursor.line += 1;
                self.state.cursor.col = 0;
                self.on_buffer_modified();
            }
            _ => {}
        }
    }

    #[allow(dead_code)]
    fn page_down(&mut self) {
        let terminal_rows = self.terminal.rows() as usize;
        let line_count = self.buffer.line_count();
        self.state.cursor.line = (self.state.cursor.line + terminal_rows).min(line_count).max(1);
        let len = self.buffer.get_line(self.state.cursor.line).len();
        self.state.cursor.col = self.state.cursor.col.min(len.saturating_sub(1));
    }

    #[allow(dead_code)]
    fn page_up(&mut self) {
        let terminal_rows = self.terminal.rows() as usize;
        self.state.cursor.line = self.state.cursor.line.saturating_sub(terminal_rows).max(1);
        let len = self.buffer.get_line(self.state.cursor.line).len();
        self.state.cursor.col = self.state.cursor.col.min(len.saturating_sub(1));
    }

    fn scroll_by(&mut self, lines: usize) {
        if self.state.cursor.line > lines {
            self.state.cursor.line -= lines;
        } else {
            self.state.cursor.line = 1;
        }
    }

    fn jump_to_matching_bracket(&mut self) {
        let line = self.state.cursor.line;
        let col = self.state.cursor.col;
        let line_str = self.buffer.get_line(line);
        
        if col >= line_str.len() {
            return;
        }
        
        let ch = line_str.chars().nth(col);
        if ch.is_none() {
            return;
        }
        let ch = ch.unwrap();
        
        let matching = match ch {
            '(' => ')',
            ')' => '(',
            '[' => ']',
            ']' => '[',
            '{' => '}',
            '}' => '{',
            _ => return,
        };
        
        let direction = if ch == matching { 1 } else { -1 };
        
        let mut count = 1;
        let mut current_line = line;
        let mut current_col = col;
        
        loop {
            if direction > 0 {
                current_col += 1;
                if current_col >= self.buffer.get_line(current_line).len() {
                    current_line += 1;
                    if current_line > self.buffer.line_count() {
                        break;
                    }
                    current_col = 0;
                }
            } else {
                if current_col == 0 {
                    if current_line == 1 {
                        break;
                    }
                    current_line -= 1;
                    current_col = self.buffer.get_line(current_line).len().saturating_sub(1);
                } else {
                    current_col -= 1;
                }
            }
            
            let current_char = self.buffer.get_line(current_line).chars().nth(current_col);
            if let Some(c) = current_char {
                if c == ch {
                    count += 1;
                } else if c == matching {
                    count -= 1;
                    if count == 0 {
                        self.state.cursor.line = current_line;
                        self.state.cursor.col = current_col;
                        return;
                    }
                }
            }
            
            if current_line > self.buffer.line_count() || current_line < 1 {
                break;
            }
        }
    }

    fn delete_char(&mut self, _register: char) {
        let line = self.state.cursor.line;
        let col = self.state.cursor.col;
        let line_str = self.buffer.get_line(line);
        
        if col >= line_str.len() {
            if line < self.buffer.line_count() {
                self.buffer.merge_with_prev_line(line + 1);
            }
        } else {
            self.buffer.delete(line, col);
        }
        
        self.dot_last_action = Some(DotAction::Delete { text: String::new(), line, col });
        self.on_buffer_modified();
    }

    async fn repeat_last_action(&mut self) {
        if let Some(action) = &self.dot_last_action {
            match action.clone() {
                DotAction::Insert { text } => {
                    for ch in text.chars() {
                        self.buffer.insert_char(self.state.cursor.line, self.state.cursor.col, ch);
                        self.state.cursor.col += 1;
                    }
                    self.on_buffer_modified();
                }
                DotAction::Delete { text, line, col } => {
                    if !text.is_empty() {
                        self.buffer.insert(line, col, &text);
                        self.on_buffer_modified();
                    }
                }
                DotAction::Change { text, line, col } => {
                    self.buffer.insert(line, col, &text);
                    self.state.cursor.line = line;
                    self.state.cursor.col = col;
                    self.on_buffer_modified();
                }
            }
        }
    }

    fn move_word_forward(&mut self) {
        let line = self.state.cursor.line;
        let col = self.state.cursor.col;
        let line_str = self.buffer.get_line(line);
        let chars: Vec<char> = line_str.chars().collect();
        
        let mut i = col;
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        
        if i < chars.len() {
            self.state.cursor.col = i;
        }
    }

    fn move_word_backward(&mut self) {
        let line = self.state.cursor.line;
        let col = self.state.cursor.col;
        let line_str = self.buffer.get_line(line);
        let chars: Vec<char> = line_str.chars().collect();
        
        if col == 0 {
            return;
        }
        
        let mut i = col - 1;
        while i > 0 && chars[i].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !chars[i - 1].is_whitespace() {
            i -= 1;
        }
        
        self.state.cursor.col = i;
    }

    fn move_word_end(&mut self) {
        let line = self.state.cursor.line;
        let col = self.state.cursor.col;
        let line_str = self.buffer.get_line(line);
        let chars: Vec<char> = line_str.chars().collect();
        
        let mut i = col;
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        
        if i > 0 && i <= chars.len() {
            self.state.cursor.col = i - 1;
        }
    }

    fn open_line(&mut self, above: bool) {
        let line = self.state.cursor.line;
        
        if above {
            self.buffer.insert(line, 0, "\n");
            self.state.cursor.line = line;
        } else {
            self.buffer.insert(line + 1, 0, "\n");
            self.state.cursor.line = line + 1;
        }
        self.state.cursor.col = 0;
        
        let prev_mode = self.state.mode;
        self.state.mode = Mode::Insert;
        self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Insert });
        
        self.dot_last_action = Some(DotAction::Insert { text: String::new() });
        self.on_buffer_modified();
    }

    fn join_lines(&mut self) {
        let line = self.state.cursor.line;
        if line >= self.buffer.line_count() {
            return;
        }
        
        let current_line = self.buffer.get_line(line);
        let _next_line = self.buffer.get_line(line + 1);
        
        self.buffer.delete_line(line + 1);
        
        if current_line.ends_with(' ') || current_line.ends_with('\t') {
        } else {
            self.buffer.insert(line, current_line.len(), " ");
        }
        
        self.dot_last_action = Some(DotAction::Change { text: String::new(), line, col: 0 });
        self.on_buffer_modified();
    }

    fn scroll_cursor_to_center(&mut self) {
        let terminal_rows = self.terminal.rows() as usize;
        let visible_rows = (terminal_rows as usize).saturating_sub(2);
        let scroll_pos = self.state.cursor.line.saturating_sub(visible_rows / 2);
        self.state.cursor.line = scroll_pos.max(1);
    }

    fn scroll_cursor_to_top(&mut self) {
        self.state.cursor.line = 1;
    }

    fn scroll_cursor_to_bottom(&mut self) {
        let terminal_rows = self.terminal.rows() as usize;
        let visible_rows = (terminal_rows as usize).saturating_sub(2);
        let line_count = self.buffer.line_count();
        self.state.cursor.line = (line_count.saturating_sub(visible_rows) + 1).max(1);
    }

    #[allow(dead_code)]
    fn toggle_line_numbers(&mut self) {
        self.state.show_line_numbers = !self.state.show_line_numbers;
    }

    fn scroll_up_one(&mut self) {
        if self.state.cursor.line > 1 {
            self.state.cursor.line -= 1;
        }
    }

    fn scroll_down_one(&mut self) {
        let line_count = self.buffer.line_count();
        if self.state.cursor.line < line_count {
            self.state.cursor.line += 1;
        }
    }

    fn show_file_info(&mut self) {
        let line_count = self.buffer.line_count();
        let _col = self.state.cursor.col + 1;
        let total = self.buffer.len_chars();
        let path = self.state.file_path.as_ref()
            .map(|p| p.to_str().unwrap_or(""))
            .unwrap_or("[No Name]");
        eprintln!("\"{}\" {} lines, {} characters", path, line_count, total);
    }
}
