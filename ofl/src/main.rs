use {
    failure::Error,
    gumdrop::Options,
    std::path::{Path, PathBuf},
    tracing::*,
    tracing_fmt::{filter::EnvFilter, FmtSubscriber},
};

mod published;
mod server;
mod website;

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
    Published(published::EnsurePublished),
    Serve(server::ServerOpts),
    Website(website::Website),
}

impl Default for Command {
    fn default() -> Self {
        Command::Serve(Default::default())
    }
}

fn main() -> Result<(), Error> {
    let config = Config::parse_args_default_or_exit();
    let level = if config.verbose { "debug" } else { "info" };
    tracing::subscriber::with_default(
        FmtSubscriber::builder()
            .with_filter(EnvFilter::new(level))
            .finish(),
        || {
            debug!("logging init'd");

            let command = config.command.unwrap_or_default();

            match command {
                Command::Published(opts) => {
                    let metadata = cargo_metadata::MetadataCommand::new()
                        .manifest_path(config.project_root.join("Cargo.toml"))
                        .current_dir(config.project_root)
                        .exec()?;
                    opts.run(metadata)
                }
                Command::Serve(opts) => opts.run_server(config.project_root),
                Command::Website(opts) => opts.run(config.project_root),
            }
        },
    )
}
