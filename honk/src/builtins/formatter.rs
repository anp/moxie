use crate::builtins::command::Command;
use starlark::values::{none::NoneType, Value};

starlark_module! { globals =>
    formatter(name: String, command: Command, affected: Value) {
        tracing::warn!(%name, ?command, "TODO implement formatters");
        Ok(Value::new(NoneType::None))
    }
}
