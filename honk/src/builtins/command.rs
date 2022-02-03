use crate::{builtins::path::HonkPath, error::Error};
use starlark::{
    environment::GlobalsBuilder,
    starlark_simple_value,
    values::{ARef, StarlarkValue, Value},
};
use starlark_module::starlark_module;
use tracing::instrument;

#[starlark_module]
pub fn register(builder: &mut GlobalsBuilder) {
    fn command(
        command: String,
        _args: Vec<Value<'_>>,
        inputs: Vec<ARef<HonkPath>>,
        outputs: Vec<ARef<HonkPath>>,
    ) -> HonkCommand {
        let args = _args.iter().map(|a| a.to_str()).collect();
        let inputs = inputs.iter().map(|i| (*i).clone()).collect();
        let outputs = outputs.iter().map(|o| (*o).clone()).collect();
        Ok(HonkCommand { command, args, inputs, outputs })
    }
}

#[derive(Clone, Debug)]
pub struct HonkCommand {
    pub command: String,
    pub args: Vec<String>,
    pub inputs: Vec<HonkPath>,
    pub outputs: Vec<HonkPath>,
}

impl HonkCommand {
    #[instrument]
    pub fn run(&self) -> Result<Output, Error> {
        // TODO read file metadata for inputs from vfs
        let output = Output {
            // TODO set working dir
            // TODO set environment
            inner: std::process::Command::new(&self.command).args(&self.args).output()?,
            command: self.to_string(),
        };
        if output.inner.status.success() {
            Ok(output)
        } else {
            Err(Error::CommandFailed(output))
        }
    }
}

starlark_simple_value!(HonkCommand);

#[starlark_module::starlark_module]
fn register_command_methods(globals: &mut GlobalsBuilder) {
    fn run(this: ARef<HonkCommand>) -> Output {
        Ok(this.run()?)
    }
}

impl StarlarkValue<'_> for HonkCommand {
    starlark::starlark_type!("command");
    declare_members!(register_command_methods);

    fn collect_repr(&self, buf: &mut String) {
        use std::fmt::Write;
        write!(buf, "{}", self).unwrap();
    }
}

impl std::fmt::Display for HonkCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "HonkCommand:")?;
        write!(f, "\t{}", &self.command)?;
        for arg in &self.args {
            write!(f, " {}", arg)?;
        }

        if !self.inputs.is_empty() {
            write!(f, "\n\tinputs:")?;
            for input in &self.inputs {
                write!(f, " {},", input)?;
            }
        }

        if !self.outputs.is_empty() {
            writeln!(f, "\n\toutputs:")?;
            for output in &self.outputs {
                write!(f, " {},", output)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Output {
    command: String,
    inner: std::process::Output,
}

impl Output {
    fn stdout(&self) -> Result<String, Error> {
        Ok(String::from_utf8(self.inner.stdout.clone())
            .map_err(|source| Error::StdoutEncoding { source, command: self.command.to_owned() })?)
    }
}

#[starlark_module::starlark_module]
fn register_output_methods(globals: &mut GlobalsBuilder) {
    fn stdout(this: ARef<Output>) -> String {
        Ok(this.stdout()?)
    }
}

starlark_simple_value!(Output);

impl StarlarkValue<'_> for Output {
    starlark::starlark_type!("Output");
    declare_members!(register_output_methods);
}
