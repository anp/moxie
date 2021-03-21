use crate::builtins::command::RefHonkCommand;
use gazebo::any::AnyLifetime;

#[derive(AnyLifetime, Clone, Debug, Default)]
pub struct Revision {}

impl Revision {
    pub fn register_formatter(&self, name: &str, command: RefHonkCommand) {
        tracing::warn!(%name, command = %&*command, "TODO implement formatters");
    }

    pub fn register_target(&self, name: &str, command: RefHonkCommand) {
        tracing::warn!(%name, command = %&*command, "TODO implement targets");
    }
}
