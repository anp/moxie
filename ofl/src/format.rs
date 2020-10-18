use anyhow::{bail, Context, Error};
use gumdrop::Options;
use std::{path::Path, process::Command};
use tracing::*;

use crate::workspace::Workspace;

#[derive(Debug, Options)]
pub struct Format {
    /// Pass the `--check` flag to rustfmt.
    check: bool,
}

impl Format {
    pub fn run(&self, project_root: impl AsRef<Path>) -> Result<(), Error> {
        let workspace = Workspace::get(project_root)?;

        workspace.ensure_rustfmt_toolchain()?;

        let mut command = Command::new("rustup");
        command
            .arg("run")
            .arg(&workspace.rustfmt_toolchain)
            .arg("rustfmt")
            .arg("--edition")
            .arg("2018");

        if self.check {
            command.arg("--check");
        }

        for member in workspace.local_members() {
            let targets = &workspace.metadata[&member].targets;
            command.args(targets.iter().map(|t| &t.src_path));
        }

        for ofl_member in workspace.ofl_members() {
            let ofl_targets = &workspace.ofl_metadata[&ofl_member].targets;
            command.args(ofl_targets.iter().map(|t| &t.src_path));
        }

        debug!({ ?command }, "running rustfmt");

        let status = command.status().context("running rustfmt command")?;
        if !status.success() {
            error!({ ?status }, "rustfmt failed");
            bail!("rustfmt failed! {:?}", status);
        }

        Ok(())
    }
}
