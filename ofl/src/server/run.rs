use failure::{bail, Error};
use gumdrop::Options;
use std::{
    path::PathBuf,
    process::Command,
    thread::{sleep, spawn},
    time::Duration,
};
use tracing::*;
use which::which;

#[derive(Debug, Options)]
pub struct RunOpts {
    /// the port to use for the ephemeral http server
    #[options(default = "8000")]
    port: u16,
    /// run once the http server is up
    #[options(free)]
    cmd: String,
    /// args to pass the command
    #[options(free)]
    args: Vec<String>,
}

impl RunOpts {
    pub fn run(self, root_path: PathBuf) -> Result<(), Error> {
        let mut server = super::ServerOpts::default();
        server.port = self.port;
        server.watch_changes = false;

        let _server = spawn(move || {
            if let Err(error) = server.run_server(root_path) {
                error!(?error, "server failed, exiting");
                std::process::abort();
            }
        });

        info!("checking server...");
        let url = format!("http://[::1]:{}", self.port);
        while reqwest::blocking::get(&url).is_err() {
            info!("server not yet ready, trying again...");
            sleep(Duration::from_secs(1));
        }

        let mut command = Command::new(which(self.cmd)?);
        command.args(self.args);

        info!(?command, "running");
        let status = command.status()?;
        info!(%status, "finished");

        if !status.success() {
            bail!("command failed");
        }

        Ok(())
    }
}
