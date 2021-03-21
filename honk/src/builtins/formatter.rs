use crate::{builtins::command::RefHonkCommand, EvaluatorExt};
use starlark::{environment::GlobalsBuilder, values::none::NoneType};

#[starlark_module::starlark_module]
pub fn register(globals: &mut GlobalsBuilder) {
    fn formatter(name: &str, command: RefHonkCommand) -> NoneType {
        ctx.revision().register_formatter(name, command);
        Ok(NoneType)
    }
}
