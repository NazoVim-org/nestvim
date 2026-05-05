use crate::plugin::{Plugin, PluginApi};
use crate::types::PluginEvent;
use std::path::Path;

#[allow(dead_code)]
pub struct LuaPlugin {
    name: String,
}

impl Plugin for LuaPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn setup(&mut self, _api: &PluginApi) {}

    fn handle_event(&mut self, _event: &PluginEvent) {}

    fn execute_command(&mut self, _cmd: &str, _args: Vec<String>) -> bool {
        false
    }
}

#[allow(dead_code)]
pub fn load_lua_plugin(path: &Path, _api: std::rc::Rc<PluginApi>) -> Result<Box<dyn Plugin>, String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    // nameをコードから抽出（簡易版）
    let name = code.lines()
        .find_map(|line| {
            let line = line.trim();
            if line.starts_with("name") && line.contains("=") {
                line.split('=')
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| format!("Missing plugin name in {}", path.display()))?;

    Ok(Box::new(LuaPlugin { name }))
}
