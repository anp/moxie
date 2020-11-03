use argh::FromArgs;
use honk::Workspace;
use std::path::PathBuf;

/// An awful billed system.
#[derive(Debug, FromArgs)]
struct HonkCli {
    /// path to the workspace root.
    #[argh(option, default = "std::env::current_dir().unwrap()")]
    workspace: PathBuf,
}

fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;
    let HonkCli { workspace } = argh::from_env();
    Workspace::open(workspace)?.run()
}
