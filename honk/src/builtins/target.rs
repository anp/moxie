use crate::{builtins::command::RefHonkCommand, EvaluatorExt};
use starlark::{environment::GlobalsBuilder, values::{Value, none::NoneType}};

#[starlark_module::starlark_module]
pub fn register(globals: &mut GlobalsBuilder) {
    fn target(name: &str, command: RefHonkCommand, deps: Option<Vec<Value<'_>>>) -> NoneType {
        ctx.revision().register_target(name, command);
        Ok(NoneType)
    }
}
