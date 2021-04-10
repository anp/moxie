use std::path::Path;
use tracing::{debug, error, info, instrument};

pub mod builtins;
pub mod error;
pub mod graph;
pub mod revision;
pub mod state;
pub mod vfs;

use crate::{error::Error, revision::EvaluatorExt, state::WorkspaceState};

pub(crate) type Result<T> = color_eyre::eyre::Result<T, Error>;

pub struct Workspace {
    state: WorkspaceState,
}

impl Workspace {
    /// The asset path used to resolve the root of a honk workspace.
    const ASSET_PATH: &'static str = "WORKSPACE.honk";

    pub fn new(root: impl AsRef<Path>) -> Self {
        let state = WorkspaceState::new(root);
        Self { state }
    }

    pub fn maintain(self) -> crate::Result<()> {
        // TODO change current directory to workspace root?
        info!("maintaining workspace");
        loop {
            if let Err(error) = self.converge() {
                error!(%error, "couldn't converge current workspace revision");
            }
            self.state.wait_for_changes();
        }
    }

    #[instrument(level = "info", skip(self), fields(root = %self.state.root().display()))]
    fn converge(&self) -> crate::Result<()> {
        debug!("constructing workspace env");
        self.state.start_new_revision();
        let _workspace_env = self.state.load(Self::ASSET_PATH)?;

        let _build = self.state.current_revision().resolve()?;
        info!("discovered targets");

        // FIXME make this an actual web viewer via http server, right?
        dump_graphviz(&_build);

        tracing::warn!("uh run some builds i guess?");

        info!("finished");
        Ok(())
    }
}

fn dump_graphviz(g: &crate::graph::ActionGraph) {
    use petgraph::dot::{Config, Dot};
    let output = Dot::with_config(g, &[Config::EdgeNoLabel]);
    println!("{}", output);
}
