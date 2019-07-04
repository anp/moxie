use std::cell::RefCell;

/// Produce a tree node to be maintained by the current component topology.
#[topo::bound]
pub fn produce<N>(node: N, with_new_as_parent: impl FnOnce())
where
    N: Node + 'static,
{
    let current_parent: &Parent<N> = &*topo::Env::expect();
    current_parent
        .node
        .borrow_mut()
        .child(topo::Id::current(), &node);
    topo::call!(
        with_new_as_parent(),
        env! {
            Parent<N> => Parent { node: RefCell::new(node), },
        }
    )
}

#[topo::bound]
pub fn produce_root<N>(new_root: N, with_new_root_as_parent: impl FnOnce())
where
    N: Node + 'static,
{
    topo::call!(
        with_new_root_as_parent(),
        env! {
            Parent<N> => Parent { node: RefCell::new(new_root), },
        }
    );
}

/// A type which can be attached to parents of its type and which can receive children.
pub trait Node {
    /// TODO
    fn child(&mut self, id: topo::Id, child: &Self);
}

struct Parent<N> {
    node: RefCell<N>,
}

impl<N> Parent<N>
where
    N: Node + 'static,
{
    // TODO handle index within children?
    // TODO handle start/end of a scope?
    fn child(&self, id: topo::Id, child: &N) {
        self.node.borrow_mut().child(id, child);
    }
}
