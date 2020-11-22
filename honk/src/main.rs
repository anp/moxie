use argh::FromArgs;
use honk::Workspace;
use std::path::PathBuf;
use tracing_subscriber::Layer;

/// An awful billed system.
#[derive(Debug, FromArgs)]
struct HonkCli {
    /// path to directory containing `workspace.honk`.
    #[argh(option, default = "std::env::current_dir().unwrap()")]
    workspace: PathBuf,
}

fn main() -> color_eyre::eyre::Result<()> {
    let subscriber = tracing_error::ErrorLayer::default()
        .with_subscriber(tracing_subscriber::fmt::fmt().pretty().finish());
    tracing::subscriber::set_global_default(subscriber)?;
    color_eyre::install()?;
    let HonkCli { workspace } = argh::from_env();
    Workspace::new(workspace).maintain()
}
