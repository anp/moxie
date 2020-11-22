use crate::builtins::command::Command;
use starlark::values::Value;

starlark_module! { globals =>
    formatter(name: String, command: Command, affected: Value) {
        todo!()
    }
}
