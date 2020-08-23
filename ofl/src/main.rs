use failure::Error;
use gumdrop::Options;
use std::path::{Path, PathBuf};
use tracing::*;
use tracing_subscriber::{filter::LevelFilter, fmt::Subscriber, Layer};

mod coverage;
mod format;
mod published;
mod server;
mod versions;
mod website;
mod workspace;

#[derive(Debug, Options)]
struct Config {
    help: bool,
    verbose: bool,
    #[options(
        default_expr = r#"Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_owned()"#
    )]
    project_root: PathBuf,
    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Options)]
enum Command {
    Coverage(coverage::Coverage),
    Published(published::EnsurePublished),
    Serve(server::ServerOpts),
    ServeThenRun(server::RunOpts),
    Website(website::Website),
    /// Format all targets in the repository, including those under `ofl/`.
    Fmt(format::Format),
    Versions(versions::Versions),
}

impl Default for Command {
    fn default() -> Self {
        Command::Serve(Default::default())
    }
}

fn main() -> Result<(), Error> {
    let config = Config::parse_args_default_or_exit();
    let level = if config.verbose { LevelFilter::DEBUG } else { LevelFilter::INFO };
    tracing::subscriber::set_global_default(level.with_subscriber(Subscriber::new())).unwrap();
    info!("logging init'd");

    let command = config.command.unwrap_or_default();

    match command {
        Command::Coverage(opts) => opts.run(config.project_root),
        Command::Fmt(opts) => opts.run(config.project_root),
        Command::Published(opts) => opts.run(config.project_root),
        Command::Serve(opts) => opts.run_server(config.project_root),
        Command::ServeThenRun(opts) => opts.run(config.project_root),
        Command::Website(opts) => opts.run(config.project_root),
        Command::Versions(opts) => opts.run(config.project_root),
    }
}
