use crate::{
    builtins::{command::HonkCommand, target::DepSet},
    graph::{ActionGraph, GraphBuilder},
};
use gazebo::any::AnyLifetime;
use parking_lot::Mutex;
use std::{collections::BTreeMap, sync::Arc};

#[derive(AnyLifetime, Clone, Debug, Default)]
pub struct Revision {
    inner: Arc<Mutex<RevisionState>>,
}

impl Revision {
    pub fn register_formatter(&self, name: &str, command: &HonkCommand) {
        self.inner.lock().register_formatter(name, command);
    }

    pub fn register_target(&self, name: &str, command: &HonkCommand, deps: &DepSet) {
        self.inner.lock().register_target(name, command, deps);
    }

    pub fn resolve(&self) -> crate::Result<ActionGraph> {
        self.inner.lock().resolve()
    }
}

#[derive(Debug, Default)]
struct RevisionState {
    formatters: BTreeMap<String, (HonkCommand, DepSet)>,
    targets: BTreeMap<String, (HonkCommand, DepSet)>,
}

impl RevisionState {
    fn register_formatter(&mut self, name: &str, command: &HonkCommand) {
        let mut command = command.clone();
        // TODO find a better way to avoid cycles in the dep graph
        command.inputs.clear();
        command.outputs.clear();
        self.formatters.insert(name.to_owned(), (command, Default::default()));
    }

    fn register_target(&mut self, name: &str, command: &HonkCommand, deps: &DepSet) {
        self.targets.insert(name.to_owned(), (command.clone(), deps.clone()));
    }

    fn resolve(&mut self) -> crate::Result<ActionGraph> {
        let mut graph = GraphBuilder::new();

        for (name, (formatter, deps)) in &self.formatters {
            let idx = graph.command(name, formatter, deps);
            graph.dep(graph.formatted(), idx);
        }

        for (name, (target, deps)) in &self.targets {
            let idx = graph.command(name, target, deps);
            graph.dep(idx, graph.formatted());
        }

        graph.build()
    }
}
