use failure::{bail, Error, ResultExt};
use gumdrop::Options;
use std::{path::Path, process::Command};
use tracing::*;

/// project code coverage
#[derive(Debug, Options)]
pub enum Coverage {
    /// collect code coverage by running a cargo command
    Collect(Collect),
}

impl Coverage {
    pub fn run(&self, project_root: impl AsRef<Path>) -> Result<(), Error> {
        match self {
            Coverage::Collect(opts) => opts.run(project_root),
        }
    }
}

/// run cargo with environment variables to collect source code coverage from
/// tests
#[derive(Debug, Options)]
pub struct Collect {
    /// the command to pass to cargo
    #[options(free)]
    args: Vec<String>,
}

impl Collect {
    /// Run cargo with the `coverage` profile and cfg enabled.
    pub fn run(&self, _project_root: impl AsRef<Path>) -> Result<(), Error> {
        let mut command = Command::new("cargo");
        command
            .env("RUSTDOCFLAGS", "-Cpanic=abort")
            .env("RUSTFLAGS", "-Zprofile -Zpanic_abort_tests -Clink-dead-code")
            .args(&self.args)
            .arg("-Zunstable-options")
            // defined in .cargo/config.toml
            .arg("--profile")
            .arg("coverage");
        debug!({ ?command }, "running");

        let status = command.status().context("running cargo command")?;
        if !status.success() {
            error!({ ?status }, "cargo failed");
            bail!("cargo failed! {:?}", status);
        }

        Ok(())
    }
}

