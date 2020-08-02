use super::{
    dep_node::{DepNode, Dependent},
    Liveness,
};
use std::{
    any::type_name,
    borrow::Borrow,
    fmt::{Debug, Formatter, Result as FmtResult},
};

/// A CacheCell represents the storage used for a particular input/output pair
/// on the heap.
pub(crate) struct CacheCell<Input, Output> {
    dep: DepNode,
    input: Input,
    output: Output,
}

impl<Input, Output> CacheCell<Input, Output> {
    pub fn new(input: Input, output: Output, dep: DepNode) -> Self {
        Self { dep, input, output }
    }

    /// Return a reference to the output if the input is equal, marking it live
    /// in the process. If get fails, returns its own `Dependent` to be used as
    /// a dependency of any queries which are invoked to re-initialize this
    /// cell.
    pub fn get<Arg>(&self, input: &Arg, dependent: Dependent) -> Result<&Output, Dependent>
    where
        Arg: PartialEq<Input> + ?Sized,
        Input: Borrow<Arg>,
    {
        self.dep.root(dependent);
        if input == &self.input { Ok(&self.output) } else { Err(self.dep.as_dependent()) }
    }

    /// Store a new input/output and mark the storage live.
    pub fn store(&mut self, input: Input, output: Output, dependent: Dependent) {
        self.dep.root(dependent);
        self.input = input;
        self.output = output;
    }

    pub fn liveness(&self) -> Liveness {
        self.dep.liveness()
    }

    pub fn update_liveness(&mut self) {
        self.dep.update_liveness();
    }

    pub fn mark_dead(&mut self) {
        self.dep.mark_dead();
    }
}

impl<Input, Output> Debug for CacheCell<Input, Output>
where
    Input: 'static,
    Output: 'static,
{
    // someday specialization might save us from these lame debug impls?
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_map()
            .entry(&"input", &type_name::<Input>())
            .entry(&"output", &type_name::<Output>())
            .finish()
    }
}
