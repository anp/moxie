use crate::builtins::command::Command;
use starlark::values::{none::NoneType, Value};

starlark_module! { globals =>
    target(name: String, command: Command) {
        tracing::warn!(%name, ?command, "TODO implement targets");
        Ok(Value::new(NoneType::None))
    }
}
