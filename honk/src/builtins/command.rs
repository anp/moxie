use crate::error::Error;
use starlark::values::{TypedValue, Value};
use tracing::instrument;

// TODO define an actual asset type that can be linked to vfs
type Asset = Value;

starlark_module! { globals =>
    command(command: String, args: Vec<Value>, inputs: Vec<Asset> = vec![]) {
        let args = args.iter().map(Value::to_str).collect();
        Ok(Value::new(Command::new(command, args, inputs)))
    }

    Command.run(this: Command) {
        Ok(Value::new(this.run()?))
    }

    Output.stdout(this: Output) {
        Ok(Value::new(this.stdout()?))
    }
}

#[derive(Clone, Debug)]
pub struct Command {
    command: String,
    args: Vec<String>,
    inputs: Vec<Asset>,
}

impl Command {
    pub fn new(command: String, args: Vec<String>, inputs: Vec<Asset>) -> Self {
        Self { command, args, inputs }
    }

    #[instrument]
    pub fn run(self) -> Result<Output, Error> {
        // TODO read file metadata for inputs from vfs
        let output = Output {
            // TODO set working dir
            // TODO set environment
            inner: std::process::Command::new(&self.command).args(&self.args).output()?,
            command: self,
        };
        if output.inner.status.success() { Ok(output) } else { Err(Error::CommandFailed(output)) }
    }
}

impl TypedValue for Command {
    type Holder = starlark::values::Immutable<Self>;

    const TYPE: &'static str = "Command";

    fn values_for_descendant_check_and_freeze(&self) -> Box<dyn Iterator<Item = Value> + '_> {
        Box::new(std::iter::empty())
    }
}

#[derive(Debug, Clone)]
pub struct Output {
    command: Command,
    inner: std::process::Output,
}

impl Output {
    fn stdout(&self) -> Result<String, Error> {
        Ok(String::from_utf8(self.inner.stdout.clone())
            .map_err(|source| Error::StdoutEncoding { source, command: self.command.clone() })?)
    }
}

impl TypedValue for Output {
    type Holder = starlark::values::Immutable<Self>;

    const TYPE: &'static str = "Output";

    fn values_for_descendant_check_and_freeze(&self) -> Box<dyn Iterator<Item = Value> + '_> {
        Box::new(std::iter::empty())
    }
}
