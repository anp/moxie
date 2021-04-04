use crate::{
    builtins::{
        command::{HonkCommand, RefHonkCommand},
        path::HonkPath,
    },
    error::Error,
};
use gazebo::any::AnyLifetime;
use parking_lot::Mutex;
use petgraph::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

#[derive(AnyLifetime, Clone, Debug, Default)]
pub struct Revision {
    inner: Arc<Mutex<RevisionState>>,
}

impl Revision {
    pub fn register_formatter(&self, name: &str, command: RefHonkCommand) {
        self.inner.lock().register_formatter(name, command);
    }

    pub fn register_target(&self, name: &str, command: RefHonkCommand) {
        self.inner.lock().register_target(name, command);
    }

    pub fn resolve(&self) -> crate::Result<ActionGraph> {
        self.inner.lock().resolve()
    }
}

#[derive(Debug, Default)]
struct RevisionState {
    formatters: BTreeMap<String, HonkCommand>,
    targets: BTreeMap<String, HonkCommand>,
}

impl RevisionState {
    fn register_formatter(&mut self, name: &str, command: RefHonkCommand) {
        let mut command = command.clone();
        // TODO find a better way to avoid cycles in the dep graph
        command.inputs.clear();
        command.outputs.clear();
        self.formatters.insert(name.to_owned(), command);
    }

    fn register_target(&mut self, name: &str, command: RefHonkCommand) {
        self.targets.insert(name.to_owned(), command.clone());
    }

    fn resolve(&mut self) -> crate::Result<ActionGraph> {
        let mut graph = GraphBuilder::new();

        for (name, formatter) in &self.formatters {
            let idx = graph.command(name, formatter);
            graph.dep(graph.formatted(), idx);
        }

        for (name, target) in &self.targets {
            let idx = graph.command(name, target);
            graph.dep(idx, graph.formatted());
        }

        graph.build()
    }
}

pub type ActionGraph = DiGraph<Action, i32>;
pub type DepGraph = DiGraph<Arc<Node>, i32>;

pub struct GraphBuilder {
    inner: DepGraph,
    indices: HashMap<Arc<Node>, NodeIndex>,
    formatted: Arc<Node>,
    pending_target_name_deps: HashMap<String, Vec<NodeIndex>>,
    target_name_to_idx: HashMap<String, NodeIndex>,
}

impl GraphBuilder {
    fn new() -> Self {
        let mut inner = DepGraph::default();
        let mut indices = HashMap::default();
        let formatted = Arc::new(Node::Formatted);
        indices.insert(formatted.clone(), inner.add_node(formatted.clone()));
        Self {
            inner,
            indices,
            formatted,
            pending_target_name_deps: Default::default(),
            target_name_to_idx: Default::default(),
        }
    }

    fn command(&mut self, name: &str, command: &HonkCommand) -> NodeIndex {
        let idx = self.action(name, &command.command, &command.args[..]);

        for input in &command.inputs {
            todo!()
            // match input {
            //     Input::File(f) => {
            //         let input = self.file(f);
            //         self.dep(idx, input);
            //     }
                // Input::Target(t) => {
                //     self.pending_target_name_deps.entry(t.to_string()).or_default().push(idx)
                // }
            // }
        }

        for output in &command.outputs {
            let output = self.file(output);
            self.dep(output, idx);
        }

        idx
    }

    fn file(&mut self, path: &HonkPath) -> NodeIndex {
        let Self { inner, indices, .. } = self;
        // TODO less allocating?
        let node = Arc::new(Node::File(path.to_owned()));
        *indices.entry(node.clone()).or_insert_with(|| inner.add_node(node))
    }

    fn action(&mut self, name: &str, command: &str, args: &[String]) -> NodeIndex {
        let Self { inner, indices, target_name_to_idx, .. } = self;
        // TODO less allocating?
        let args = args.iter().map(|a| a.to_string()).collect();
        let node = Arc::new(Node::Action(Action {
            name: name.to_owned(),
            command: command.to_owned(),
            args,
        }));
        *indices.entry(node.clone()).or_insert_with(|| {
            let idx = inner.add_node(node);
            target_name_to_idx.insert(name.to_owned(), idx);
            idx
        })
    }

    fn formatted(&self) -> NodeIndex {
        self.indices[&self.formatted]
    }

    fn dep(&mut self, from: NodeIndex, to: NodeIndex) {
        self.inner.update_edge(from, to, 0);
    }

    fn build(mut self) -> crate::Result<ActionGraph> {
        self.drain_pending()?;
        let graph = rewrite_file_edges(self.inner);

        let num_components = petgraph::algo::connected_components(&graph);
        if num_components != 1 {
            Err(Error::GraphIsSplit { num_components })
        } else if petgraph::algo::is_cyclic_directed(&graph) {
            Err(Error::GraphContainsCycles)
        } else {
            Ok(graph)
        }
    }

    fn drain_pending(&mut self) -> crate::Result<()> {
        for (target, deps) in self.pending_target_name_deps.drain().collect::<Vec<_>>() {
            let target = if let Some(t) = self.target_name_to_idx.get(&target) {
                *t
            } else {
                return Err(Error::GraphResolutionFailure { target });
            };
            for dep in deps {
                self.dep(dep, target);
            }
        }

        Ok(())
    }
}

fn rewrite_file_edges(deps: DepGraph) -> ActionGraph {
    let mut actions = ActionGraph::new();

    // FIXME this leaves an empty graph lol
    // graph.retain_nodes(|this, idx| !this[idx].is_file());
    actions
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Node {
    /// A special node used to schedule all formatters before anything that relies on their output.
    Formatted,
    /// A file in the build graph.
    File(HonkPath),
    /// A command to run in the build graph.
    Action(Action),
}

impl Node {
    fn is_file(&self) -> bool {
        matches!(self, Self::File(..))
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Formatted => "FORMATTER BARRIER".fmt(f),
            Self::File(p) => write!(f, "file:{}", p),
            Self::Action(a) => write!(f, "{}", a),
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Action {
    name: String,
    command: String,
    args: Vec<String>,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "action:{}", &self.name)
    }
}
