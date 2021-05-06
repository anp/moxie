use crate::{
    builtins::{command::HonkCommand, path::HonkPath, target::DepSet},
    error::Error,
};
use petgraph::prelude::*;
use std::{collections::HashMap, sync::Arc};

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
    pub fn new() -> Self {
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

    pub fn command(&mut self, name: &str, command: &HonkCommand, deps: &DepSet) -> NodeIndex {
        let idx = self.action(
            name,
            &command.command,
            &command.args[..],
            &command.inputs[..],
            &command.outputs[..],
            deps,
        );

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

    pub fn file(&mut self, path: &HonkPath) -> NodeIndex {
        let Self { inner, indices, .. } = self;
        // TODO less allocating?
        let node = Arc::new(Node::File(path.to_owned()));
        *indices.entry(node.clone()).or_insert_with(|| inner.add_node(node))
    }

    pub fn action(
        &mut self,
        name: &str,
        command: &str,
        args: &[String],
        inputs: &[HonkPath],
        outputs: &[HonkPath],
        deps: &DepSet,
    ) -> NodeIndex {
        let Self { inner, indices, target_name_to_idx, .. } = self;
        // TODO less allocating?
        let args = args.iter().map(|a| a.to_string()).collect();
        let node = Arc::new(Node::Action(Action {
            name: name.to_owned(),
            command: command.to_owned(),
            args,
            inputs: inputs.to_vec(),
            outputs: outputs.to_vec(),
        }));
        let idx = *indices.entry(node.clone()).or_insert_with(|| {
            let idx = inner.add_node(node);
            target_name_to_idx.insert(name.to_owned(), idx);
            idx
        });

        for dep in deps {
            self.pending_target_name_deps.entry(dep.to_string()).or_default().push(idx);
        }

        idx
    }

    pub fn formatted(&self) -> NodeIndex {
        self.indices[&self.formatted]
    }

    pub fn dep(&mut self, from: NodeIndex, to: NodeIndex) {
        self.inner.update_edge(from, to, 0);
    }

    pub fn build(mut self) -> crate::Result<ActionGraph> {
        self.drain_pending()?;
        let graph = self.collapse_non_action_edges()?;

        let num_components = petgraph::algo::connected_components(&graph);
        if num_components != 1 {
            Err(Error::GraphIsSplit { num_components })
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

    fn collapse_non_action_edges(&mut self) -> crate::Result<ActionGraph> {
        // for each non action edge, get all ins and all outs, make edges between
        for i in self.inner.node_indices() {
            if matches!(&*self.inner[i], Node::Action(..)) {
                continue;
            }

            let mut in_neighbors = vec![];
            add_all_action_indices(&self.inner, i, &mut in_neighbors, Direction::Incoming);

            let mut out_neighbors = vec![];
            add_all_action_indices(&self.inner, i, &mut out_neighbors, Direction::Outgoing);

            for in_neighbor in in_neighbors {
                for out_neighbor in &out_neighbors {
                    self.inner.add_edge(in_neighbor, *out_neighbor, 0);
                }
            }
        }

        // produce a new graph without any file/formatted edges
        let graph = self.inner.filter_map(
            |_, n| match &**n {
                Node::Action(a) => Some(a.clone()),
                _ => None,
            },
            |_, e| Some(*e),
        );

        Ok(graph)
    }
}

fn add_all_action_indices(
    graph: &DepGraph,
    node: NodeIndex,
    neighbors: &mut Vec<NodeIndex>,
    dir: Direction,
) {
    for neighbor in graph.neighbors_directed(node, dir) {
        match &*graph[neighbor] {
            Node::Action(..) => neighbors.push(neighbor),
            _ => add_all_action_indices(graph, neighbor, neighbors, dir),
        }
    }
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

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Formatted => "FORMATTER BARRIER".fmt(f),
            Self::File(p) => write!(f, "file:{}", p),
            Self::Action(a) => write!(f, "{}", a),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Action {
    name: String,
    command: String,
    args: Vec<String>,
    inputs: Vec<HonkPath>,
    outputs: Vec<HonkPath>,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "action:{}", &self.name)
    }
}
