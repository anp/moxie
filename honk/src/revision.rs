use crate::builtins::command::RefHonkCommand;
use gazebo::any::AnyLifetime;
use parking_lot::Mutex;
use std::sync::Arc;

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
struct RevisionState {}

impl RevisionState {
    fn register_formatter(&mut self, name: &str, command: RefHonkCommand) {
        tracing::warn!(%name, command = %&*command, "TODO implement formatters");
    }

    fn register_target(&mut self, name: &str, command: RefHonkCommand) {
        tracing::warn!(%name, command = %&*command, "TODO implement targets");
    }
}
