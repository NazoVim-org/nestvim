// Plugin system - simplified for now
// Will be fully implemented after core editor works

pub struct PluginManager;

impl PluginManager {
    pub fn new() -> Self {
        Self
    }
    
    pub fn load_all(&mut self) -> Result<(), String> {
        // TODO: implement plugin loading
        Ok(())
    }
    
    pub fn emit(&self, _event: crate::types::PluginEvent) {
        // TODO: emit events to plugins
    }
    
    pub fn handle_key(&self, _mode: crate::types::Mode, _key: char) -> bool {
        false
    }
    
    pub fn execute_command(&self, _cmd: &str) -> bool {
        false
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
