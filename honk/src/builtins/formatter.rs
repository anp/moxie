use crate::builtins::command::RefHonkCommand;
use starlark::{environment::GlobalsBuilder, values::none::NoneType};

#[starlark_module::starlark_module]
pub fn register(globals: &mut GlobalsBuilder) {
    fn formatter(name: String, command: RefHonkCommand) -> NoneType {
        tracing::warn!(%name, command = %&*command, "TODO implement formatters");
        Ok(NoneType)
    }
}
