use crate::builtins::command::Command;
use starlark::values::{none::NoneType, Value};

starlark_module! { globals =>
    target(
        name: String,
        command: Command,
        inputs: Value,
        outputs: Value = Value::new(NoneType::None)
    ) {
        tracing::warn!(%name, ?command, "TODO implement targets");
        Ok(Value::new(NoneType::None))
    }
}
