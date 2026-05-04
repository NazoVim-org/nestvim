use crate::types::{Mode, PluginEvent};
use crate::plugin::interface::NazoPlugin;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::fs;

pub struct PluginManager {
    plugins: Vec<NazoPlugin>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn load_all(&mut self) -> Result<(), String> {
        let plugins_dir = Path::new("plugins");
        if !plugins_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(plugins_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            
            if let Some(ext) = path.extension() {
                let is_plugin = if cfg!(target_os = "macos") {
                    ext == "dylib"
                } else if cfg!(target_os = "linux") {
                    ext == "so"
                } else {
                    false
                };
                
                if is_plugin {
                    match NazoPlugin::load_plugin(&path) {
                        Ok(plugin) => {
                            let name = unsafe {
                                CStr::from_ptr(plugin.name())
                                    .to_string_lossy()
                                    .into_owned()
                            };
                            tracing::info!("Loaded plugin: {}", name);
                            self.plugins.push(plugin);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load plugin {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    pub fn emit(&self, event: PluginEvent) {
        match event {
            PluginEvent::ModeChange { from, to } => {
                let from_mode: crate::plugin::interface::Mode = from.into();
                let to_mode: crate::plugin::interface::Mode = to.into();
                for plugin in &self.plugins {
                    plugin.on_mode_change(from_mode, to_mode);
                }
            }
            PluginEvent::BufferChange => {
                for plugin in &self.plugins {
                    plugin.on_buffer_change();
                }
            }
            _ => {}
        }
    }

    pub fn handle_key(&self, mode: Mode, key: char) -> bool {
        let key_str = CString::new(key.to_string()).unwrap();
        let mode_val: crate::plugin::interface::Mode = mode.into();
        for plugin in &self.plugins {
            if plugin.on_key(mode_val, key_str.as_ptr()) {
                return true;
            }
        }
        false
    }

    pub fn execute_command(&self, cmd: &str) -> bool {
        let cmd_str = CString::new(cmd).unwrap();
        for plugin in &self.plugins {
            if plugin.execute_command(cmd_str.as_ptr()) {
                return true;
            }
        }
        false
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
