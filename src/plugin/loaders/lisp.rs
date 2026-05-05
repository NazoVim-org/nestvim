use crate::plugin::{Plugin, PluginApi};
use crate::types::PluginEvent;
use std::path::Path;
use std::rc::Rc;

pub struct LispPlugin {
    name: String,
}

impl Plugin for LispPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn setup(&mut self, _api: &PluginApi) {}

    fn handle_event(&mut self, _event: &PluginEvent) {}

    fn execute_command(&mut self, _cmd: &str, _args: Vec<String>) -> bool {
        false
    }
}

pub fn load_lisp_plugin(path: &Path, _api: Rc<PluginApi>) -> Result<Box<dyn Plugin>, String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    // Simple parser: find (name "xxx") in code
    let name = code.lines()
        .find_map(|line| {
            let line = line.trim();
            if line.contains("(name") {
                let start = line.find('"')?;
                let end = line.rfind('"')?;
                if end > start {
                    Some(line[start+1..end].to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .ok_or_else(|| format!("Missing plugin name in {}", path.display()))?;

    Ok(Box::new(LispPlugin { name }))
}
