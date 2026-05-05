mod api;
pub mod loaders;

pub use api::{Plugin, PluginApi};

use crate::types::PluginEvent;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    api: Rc<PluginApi>,
}

impl PluginManager {
    pub fn new() -> Self {
        let api = Rc::new(PluginApi {
            commands: Rc::new(RefCell::new(HashMap::new())),
            event_handlers: Rc::new(RefCell::new(HashMap::new())),
            log_fn: Box::new(|msg| eprintln!("[plugin] {}", msg)),
        });

        Self {
            plugins: Vec::new(),
            api,
        }
    }

    pub fn load_all(&mut self) -> Result<(), String> {
        let plugins_dir = std::path::Path::new("src/plugins");
        if !plugins_dir.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(plugins_dir)
            .map_err(|e| format!("Failed to read plugins dir: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
            let path = entry.path();
                let ext = path.extension();
                if ext == Some(std::ffi::OsStr::new("lua")) {
                    // Lua loader temporarily disabled
                    eprintln!("[plugin] Lua loader disabled");
                } else if ext == Some(std::ffi::OsStr::new("lisp")) {
                    match crate::plugin::loaders::lisp::load_lisp_plugin(&path, self.api.clone()) {
                        Ok(plugin) => {
                            let name = plugin.name().to_string();
                            self.add_plugin(plugin);
                            eprintln!("[plugin] Loaded Lisp plugin: {}", name);
                        }
                        Err(e) => eprintln!("[plugin] Failed to load {}: {}", path.display(), e),
                    }
                }
        }

        Ok(())
    }

    pub fn emit(&mut self, event: PluginEvent) {
        for plugin in &mut self.plugins {
            plugin.handle_event(&event);
        }

        let handlers = self.api.event_handlers.borrow();
        let event_name = match &event {
            PluginEvent::ModeChange { .. } => "mode_change",
            PluginEvent::BufferChange => "buffer_change",
            PluginEvent::Key { .. } => "key",
            PluginEvent::BufferSave { .. } => "buffer_save",
        };
        if let Some(event_handlers) = handlers.get(event_name) {
            for handler in event_handlers {
                handler(&event);
            }
        }
    }

    pub fn execute_command(&mut self, cmd: &str) -> bool {
        for plugin in &mut self.plugins {
            if plugin.execute_command(cmd, vec![]) {
                return true;
            }
        }

        let commands = self.api.commands.borrow();
        if let Some(f) = commands.get(cmd) {
            f(vec![]);
            true
        } else {
            false
        }
    }

    pub fn add_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn api(&self) -> &Rc<PluginApi> {
        &self.api
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
