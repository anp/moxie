use crate::builtins::{
    command::{HonkCommand, RefHonkCommand},
    path::HonkPath,
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

    pub fn resolve(&self) -> crate::Result<BuildGraph> {
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

    fn resolve(&mut self) -> crate::Result<BuildGraph> {
        let mut graph = BuildGraph::new();

        for (name, formatter) in &self.formatters {
            let idx = graph.command(name, formatter);
            graph.dep(graph.formatted(), idx);
        }

        for (name, target) in &self.targets {
            let idx = graph.command(name, target);
            graph.dep(idx, graph.formatted());
        }

        // TODO validate the dep graph?

        Ok(graph)
    }
}

pub struct BuildGraph {
    inner: DiGraph<Arc<Node>, ()>,
    indices: HashMap<Arc<Node>, NodeIndex>,
    formatted: Arc<Node>,
}

impl BuildGraph {
    fn new() -> Self {
        let mut inner = DiGraph::default();
        let mut indices = HashMap::default();
        let formatted = Arc::new(Node::Formatted);
        indices.insert(formatted.clone(), inner.add_node(formatted.clone()));
        Self { inner, indices, formatted }
    }

    fn command(&mut self, name: &str, command: &HonkCommand) -> NodeIndex {
        let idx = self.action(name, &command.command, &command.args[..]);

        for input in &command.inputs {
            let input = self.file(input);
            self.dep(idx, input);
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
        let Self { inner, indices, .. } = self;
        // TODO less allocating?
        let args = args.iter().map(|a| a.to_string()).collect();
        let node =
            Arc::new(Node::Action { name: name.to_owned(), command: command.to_owned(), args });
        *indices.entry(node.clone()).or_insert_with(|| inner.add_node(node))
    }

    fn formatted(&self) -> NodeIndex {
        self.indices[&self.formatted]
    }

    fn dep(&mut self, from: NodeIndex, to: NodeIndex) {
        self.inner.update_edge(from, to, ());
    }
}

impl std::fmt::Debug for BuildGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use petgraph::dot::{Config, Dot};
        Dot::with_config(&self.inner, &[Config::EdgeNoLabel]).fmt(f)
    }
}

#[derive(Eq, Hash, PartialEq)]
enum Node {
    /// A special node used to schedule all formatters before anything that relies on their output.
    Formatted,
    /// A file in the build graph.
    File(HonkPath),
    /// A command to run in the build graph.
    Action { name: String, command: String, args: Vec<String> },
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Formatted => "FORMATTER BARRIER".fmt(f),
            Self::File(p) => write!(f, "file:{}", p),
            Self::Action { name, .. } => write!(f, "action:{}", name),
        }
    }
}
