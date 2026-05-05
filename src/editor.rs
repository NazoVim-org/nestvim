use crate::buffer::TextBuffer;
use crate::highlight::Highlighter;
use crate::plugin::PluginManager;
use crate::renderer::Renderer;
use crate::terminal::Terminal;
use crate::types::{EditorState, Mode, PluginEvent};
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
    state: EditorState,
    running: bool,
    last_highlight_mod_count: usize,
    needs_render: bool,
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
        
        let state = EditorState {
            mode: Mode::Normal,
            cursor: crate::types::Position { line: 1, col: 0 },
            file_path: buffer.file_path.clone(),
            dirty: false,
            command_buffer: String::new(),
        };
        
        Ok(Self {
            terminal,
            buffer,
            highlighter,
            renderer,
            plugin_manager,
            state,
            running: false,
            last_highlight_mod_count: 0,
            needs_render: true,
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
        match self.state.mode {
            Mode::Normal => self.handle_normal(key).await,
            Mode::Insert => self.handle_insert(key).await,
            Mode::Command => self.handle_command(key).await,
        }
    }
    
    async fn handle_normal(&mut self, key: KeyCode) {
        let line_count = self.buffer.line_count();
        
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
            _ => {}
        }
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
