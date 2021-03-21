use crate::builtins::command::{HonkCommand, RefHonkCommand};
use gazebo::any::AnyLifetime;
use parking_lot::Mutex;
use std::{collections::BTreeMap, sync::Arc};

#[derive(AnyLifetime, Clone, Debug, Default)]
pub struct Revision {
    inner: Arc<Mutex<RevisionState>>,
}

impl Revision {
    pub fn register_formatter(&self, name: &str, command: RefHonkCommand) {
        self.inner.lock().register_formatter(name, command);
    }

    pub fn register_target(&self, name: &str, command: RefHonkCommand) {
        self.inner.lock().register_target(name, command);
    }
}

#[derive(Debug, Default)]
struct RevisionState {
    formatters: BTreeMap<String, HonkCommand>,
    targets: BTreeMap<String, HonkCommand>,
}

impl RevisionState {
    fn register_formatter(&mut self, name: &str, command: RefHonkCommand) {
        self.formatters.insert(name.to_owned(), command.clone());
    }

    fn register_target(&mut self, name: &str, command: RefHonkCommand) {
        self.targets.insert(name.to_owned(), command.clone());
    }
}
