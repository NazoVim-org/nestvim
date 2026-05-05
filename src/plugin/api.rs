use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::types::PluginEvent;

pub type CommandFn = Box<dyn Fn(Vec<String>)>;
pub type EventFn = Box<dyn Fn(&PluginEvent)>;

pub struct PluginApi {
    pub commands: Rc<RefCell<HashMap<String, CommandFn>>>,
    pub event_handlers: Rc<RefCell<HashMap<String, Vec<EventFn>>>>,
    pub log_fn: Box<dyn Fn(&str)>,
}

impl PluginApi {
    pub fn add_command(&self, name: String, f: CommandFn) {
        self.commands.borrow_mut().insert(name, f);
    }

    pub fn on(&self, event: String, f: EventFn) {
        self.event_handlers.borrow_mut().entry(event).or_insert_with(Vec::new).push(f);
    }

    pub fn log(&self, msg: &str) {
        (self.log_fn)(msg);
    }
}

pub trait Plugin {
    fn name(&self) -> &str;
    fn setup(&mut self, api: &PluginApi);
    fn handle_event(&mut self, event: &PluginEvent);
    fn execute_command(&mut self, cmd: &str, args: Vec<String>) -> bool;
}
