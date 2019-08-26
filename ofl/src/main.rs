use {
    failure::Error,
    gumdrop::Options,
    std::path::Path,
    tracing::*,
    tracing_fmt::{filter::EnvFilter, FmtSubscriber},
};

mod published;
mod server;
mod website;

#[derive(Debug, Options)]
struct Config {
    help: bool,
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
    tracing::subscriber::with_default(
        FmtSubscriber::builder()
            .with_filter(EnvFilter::new("debug"))
            .finish(),
        || {
            debug!("logging init'd");

            let root_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
            let config = Config::parse_args_default_or_exit();
            let command = config.command.unwrap_or_default();

            match command {
                Command::Published(opts) => {
                    let metadata = cargo_metadata::MetadataCommand::new()
                        .manifest_path(root_path.join("Cargo.toml"))
                        .current_dir(root_path)
                        .exec()?;
                    opts.run(metadata)
                }
                Command::Serve(opts) => opts.run_server(root_path.to_path_buf()),
                Command::Website(opts) => opts.run(root_path),
            }
        },
    )
}
