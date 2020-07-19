use super::{dep_node::DepNode, Gc, Liveness};
use std::{
    any::type_name,
    borrow::Borrow,
    fmt::{Debug, Formatter, Result as FmtResult},
};

/// A CacheCell represents the storage used for a particular input/output pair
/// on the heap.
pub struct CacheCell<Input, Output> {
    dep: DepNode,
    input: Input,
    output: Output,
}

impl<Input, Output> CacheCell<Input, Output> {
    pub fn new(input: Input, output: Output) -> Self {
        Self { dep: DepNode::new(), input, output }
    }

    /// Return a reference to the output if the input is equal, marking it live
    /// in the process.
    pub fn get<Arg>(&self, input: &Arg) -> Option<&Output>
    where
        Arg: PartialEq<Input> + ?Sized,
        Input: Borrow<Arg>,
    {
        if input == &self.input {
            self.dep.mark_live();
            Some(&self.output)
        } else {
            None
        }
    }

    /// Store a new input/output and mark the storage live.
    pub fn store(&mut self, input: Input, output: Output) {
        self.dep.mark_live();
        self.input = input;
        self.output = output;
    }
}

impl<Input, Output> Gc for CacheCell<Input, Output>
where
    Input: 'static,
    Output: 'static,
{
    fn sweep(&mut self) -> Liveness {
        self.dep.sweep()
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
