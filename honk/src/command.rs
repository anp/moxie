use crate::error::Error;
use starlark::{
    starlark_module,
    values::{TypedValue, Value},
};
use tracing::instrument;

// TODO submit an upstream patch to use $crate in all these macros
use starlark::{
    starlark_fun, starlark_parse_param_type, starlark_signature, starlark_signature_extraction,
    starlark_signatures,
};

starlark_module! { register_commands =>
    command(command: String, args: Vec<String>) {
        Ok(Value::new(Command::new(command, args)))
    }

    Command.run(this: Command) {
        Ok(Value::new(this.run()?))
    }
}

#[derive(Clone, Debug)]
pub struct Command {
    command: String,
    args: Vec<String>,
}

impl Command {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self { command, args }
    }

    #[instrument]
    pub fn run(self) -> Result<Output, Error> {
        // TODO set working dir
        // TODO set environment
        let inner = std::process::Command::new(self.command).args(self.args).output()?;
        Ok(Output { inner })
    }
}

impl TypedValue for Command {
    type Holder = starlark::values::Immutable<Self>;

    const TYPE: &'static str = "Command";

    fn values_for_descendant_check_and_freeze(&self) -> Box<dyn Iterator<Item = Value> + '_> {
        Box::new(std::iter::empty())
    }
}

#[derive(Clone)]
pub struct Output {
    // TODO
    inner: std::process::Output,
}

impl TypedValue for Output {
    type Holder = starlark::values::Immutable<Self>;

    const TYPE: &'static str = "Output";

    fn values_for_descendant_check_and_freeze(&self) -> Box<dyn Iterator<Item = Value> + '_> {
        Box::new(std::iter::empty())
    }
}
