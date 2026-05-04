use crate::buffer::TextBuffer;
use crate::highlight::Highlighter;
use crate::plugin::PluginManager;
use crate::renderer::Renderer;
use crate::terminal::Terminal;
use crate::types::{EditorState, Mode, PluginEvent};
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures::StreamExt;
use std::io::{self, Write};
use tokio::time::{interval, Duration};

pub struct Editor {
    terminal: Terminal,
    buffer: TextBuffer,
    highlighter: Highlighter,
    renderer: Renderer,
    plugin_manager: PluginManager,
    state: EditorState,
    running: bool,
    command_buffer: String,
    cleaned: bool,
}

impl Editor {
    pub async fn new(file_path: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut terminal = Terminal::new()?;
        
        // Check if we're in a terminal
        if !atty::is(atty::Stream::Stdin) {
            return Err("Not running in a terminal. Please run nestvim in a real terminal.".into());
        }
        
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
        if let Err(e) = highlighter.init().await {
            eprintln!("Warning: Highlighter init failed: {}", e);
        }
        
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
        };
        
        Ok(Self {
            terminal,
            buffer,
            highlighter,
            renderer,
            plugin_manager,
            state,
            running: false,
            command_buffer: String::new(),
            cleaned: false,
        })
    }
    
    pub async fn run(&mut self) -> io::Result<()> {
        self.running = true;
        
        // Initial render
        self.highlighter.update(&self.buffer.to_string(), self.state.file_path.as_deref()).await;
        self.renderer.render(&self.terminal, &self.buffer, &self.state);
        
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
                        }
                        _ => {}
                    }
                }
                _ = tick_interval.tick() => {
                    // Periodic update if needed
                }
            }
            
            self.highlighter.update(&self.buffer.to_string(), self.state.file_path.as_deref()).await;
            self.renderer.render(&self.terminal, &self.buffer, &self.state);
        }
        
        self.cleanup();
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
            }
            KeyCode::Char('l') => {
                let line_len = self.buffer.get_line(self.state.cursor.line).len();
                self.state.cursor.col = (self.state.cursor.col + 1).min(line_len.saturating_sub(1));
            }
            KeyCode::Char('j') => {
                self.state.cursor.line = (self.state.cursor.line + 1).min(line_count);
                let len = self.buffer.get_line(self.state.cursor.line).len();
                self.state.cursor.col = self.state.cursor.col.min(len.saturating_sub(1));
            }
            KeyCode::Char('k') => {
                self.state.cursor.line = self.state.cursor.line.saturating_sub(1).max(1);
                let len = self.buffer.get_line(self.state.cursor.line).len();
                self.state.cursor.col = self.state.cursor.col.min(len.saturating_sub(1));
            }
            KeyCode::Char('i') => {
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Insert;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Insert });
            }
            KeyCode::Char(':') => {
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Command;
                self.command_buffer.clear();
                let _ = self.terminal.write_status(":");
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Command });
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
            }
            KeyCode::Backspace => {
                if self.state.cursor.col > 0 {
                    self.buffer.delete(self.state.cursor.line, self.state.cursor.col - 1);
                    self.state.cursor.col -= 1;
                    self.plugin_manager.emit(PluginEvent::BufferChange);
                    self.highlighter.mark_dirty();
                } else if self.state.cursor.line > 1 {
                    let new_col = self.buffer.merge_with_prev_line(self.state.cursor.line);
                    self.state.cursor.line -= 1;
                    self.state.cursor.col = new_col;
                    self.plugin_manager.emit(PluginEvent::BufferChange);
                    self.highlighter.mark_dirty();
                }
            }
            KeyCode::Enter => {
                self.buffer.insert(self.state.cursor.line, self.state.cursor.col, "\n");
                self.state.cursor.line += 1;
                self.state.cursor.col = 0;
                self.plugin_manager.emit(PluginEvent::BufferChange);
                self.highlighter.mark_dirty();
            }
            KeyCode::Char(c) => {
                self.buffer.insert_char(self.state.cursor.line, self.state.cursor.col, c);
                self.state.cursor.col += 1;
                self.plugin_manager.emit(PluginEvent::BufferChange);
                self.highlighter.mark_dirty();
            }
            _ => {}
        }
    }
    
    async fn handle_command(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                let cmd = self.command_buffer.trim().to_string();
                self.command_buffer.clear();
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Normal;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Normal });
                
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
                self.command_buffer.clear();
                let prev_mode = self.state.mode;
                self.state.mode = Mode::Normal;
                self.plugin_manager.emit(PluginEvent::ModeChange { from: prev_mode, to: Mode::Normal });
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
                let mut stdout = std::io::stdout();
                let _ = write!(stdout, "{}", c);
                let _ = stdout.flush();
            }
            _ => {}
        }
    }
    
    fn cleanup(&mut self) {
        if self.cleaned {
            return;
        }
        self.cleaned = true;
        let _ = self.terminal.disable_raw_mode();
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        self.cleanup();
    }
}
