use crate::builtins::command::RefHonkCommand;
use starlark::{environment::GlobalsBuilder, values::none::NoneType};

#[starlark_module::starlark_module]
pub fn register(globals: &mut GlobalsBuilder) {
    fn target(name: String, command: RefHonkCommand) -> NoneType {
        tracing::warn!(%name, command = %&*command, "TODO implement targets");
        Ok(NoneType)
    }
}
