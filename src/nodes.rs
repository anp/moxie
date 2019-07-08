use {
    crate::memo::*,
    std::{cell::RefCell, collections::HashMap, rc::Rc},
};

/// A type which can be attached to parents of its type and which can receive children.
pub trait Node {
    /// A handle returned by mounting a child to this node. The handle will be `Drop`'d when the
    /// node is no longer mounted.
    type MountHandle;

    /// Mount a new child to this node, returning a [`Node::MountHandle`].
    fn child(&mut self, child: &Self) -> Self::MountHandle;
}

enum Liveness {
    Live,
    Dead,
}

struct NodeMount<N: Node> {
    node: N,
    mounts: HashMap<topo::Id, (Liveness, N::MountHandle)>,
}

impl<N> NodeMount<N>
where
    N: Node,
{
    fn new(node: N) -> Self {
        Self {
            node,
            mounts: HashMap::new(),
        }
    }

    fn end_children(&mut self) {
        // only keep those things that still live
        self.mounts.retain(|_, (lness, _)| match lness {
            Liveness::Live => true,
            Liveness::Dead => false,
        });
        // default all of our nodes to "dying" on the next round
        self.mounts
            .iter_mut()
            .for_each(|(_, (ref mut lness, _))| *lness = Liveness::Dead);
    }

    fn child(&mut self, id: topo::Id, child: &N) {
        self.mounts
            .insert(id, (Liveness::Live, self.node.child(child)));
    }
}

struct Parent<N: Node> {
    mounted: Rc<RefCell<Option<NodeMount<N>>>>,
}

impl<N> Parent<N>
where
    N: Node + 'static,
{
    fn new() -> Self {
        Self {
            mounted: Rc::new(RefCell::new(None)),
        }
    }

    fn set(&self, node: N) {
        std::mem::replace(&mut *self.mounted.borrow_mut(), Some(NodeMount::new(node)));
    }

    fn end_children(&self) {
        self.mounted.borrow_mut().as_mut().unwrap().end_children();
    }

    // TODO handle index within children?
    // TODO handle start/end of a scope?
    fn child(&self, id: topo::Id, child: &N) {
        self.mounted.borrow_mut().as_mut().unwrap().child(id, child);
    }
}

impl<N: Node> Clone for Parent<N> {
    fn clone(&self) -> Self {
        Self {
            mounted: self.mounted.clone(),
        }
    }
}

/// Produce a node without attaching it to the `Parent` in its environment.
#[topo::bound]
pub fn produce_without_attaching<N>(new_root: N, with_new_root_as_parent: impl FnOnce())
where
    N: Node + 'static,
{
    let parent = once!(|| Parent::new());
    parent.set(new_root);

    let on_end = parent.clone();
    scopeguard::defer!(on_end.end_children());

    topo::call!(
        with_new_root_as_parent(),
        env! {
            Parent<N> => parent,
        }
    );
}

/// Produce a tree node to be maintained by the current component topology. Panics
/// if no compatible `Parent` is found in the [`topo::Env`].
#[topo::bound]
pub fn produce<N>(node: N, with_new_as_parent: impl FnOnce())
where
    N: Node + 'static,
{
    let current_parent: &Parent<N> = &*topo::Env::expect();
    current_parent.child(topo::Id::current(), &node);
    produce_without_attaching!(node, with_new_as_parent);
}
