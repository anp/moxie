use crate::{builtins::command::HonkCommand, EvaluatorExt};
use starlark::{
    environment::GlobalsBuilder,
    values::{none::NoneType, ARef},
};

#[starlark_module::starlark_module]
pub fn register(globals: &mut GlobalsBuilder) {
    fn formatter(name: &str, command: ARef<HonkCommand>) -> NoneType {
        ctx.revision().register_formatter(name, &*command);
        Ok(NoneType)
    }
}
