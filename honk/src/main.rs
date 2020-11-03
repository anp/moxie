use argh::FromArgs;
use honk::Workspace;
use std::path::PathBuf;

/// An awful billed system.
#[derive(Debug, FromArgs)]
struct HonkCli {
    /// path to `workspace.honk`.
    #[argh(option, default = "default_workspace_path()")]
    workspace: PathBuf,
}

fn default_workspace_path() -> PathBuf {
    std::env::current_dir().unwrap().join("workspace.honk")
}

fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::fmt::fmt().pretty().init();
    color_eyre::install()?;
    let HonkCli { workspace } = argh::from_env();
    Workspace::new(workspace).maintain()
}
