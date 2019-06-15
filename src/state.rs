use {
    crate::{memo::*, LoopWaker, Revision},
    parking_lot::Mutex,
    std::{
        ops::Deref,
        sync::{Arc, Weak},
    },
    topo::topo,
};

// TODO state tests

/// Bind a state variable to this callsite.
#[topo]
// TODO: proc macro should allow topo functions to declare `arg` optional with a default to
// which the macro can desugar invocations, so you can pass a init closure only.
// TODO: move arg after initializer
pub fn state<Arg, Init, Output>(arg: Arg, initializer: Init) -> (Commit<Output>, Key<Output>)
where
    Arg: PartialEq + Send + Sync + 'static,
    Output: Send + Sync + 'static,
    for<'a> Init: FnOnce(&'a Arg) -> Output,
{
    static ERR: &str = "`state` must be called within a moxie runloop!";
    let current_revision = Revision::current();

    let root: Arc<Mutex<Cell<Output>>> = memo!(arg, |a| {
        let waker = topo::env::get::<LoopWaker>().expect(ERR).to_owned();
        let cell = Cell {
            last_rooted: current_revision,
            current: Commit {
                revision: current_revision,
                inner: Arc::new(initializer(a)),
            },
            pending: None,
            waker,
        };

        Arc::new(Mutex::new(cell))
    });

    let commit = {
        let mut rooted = root.lock();
        rooted.last_rooted = current_revision;
        rooted.read()
    };

    let key = Key {
        cell: Arc::downgrade(&root),
    };

    (commit, key)
}

#[derive(Debug, Eq, PartialEq)]
pub struct Commit<State> {
    revision: Revision,
    inner: Arc<State>,
}

impl<State> Clone for Commit<State> {
    fn clone(&self) -> Self {
        Self {
            revision: self.revision,
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<State> Deref for Commit<State> {
    type Target = State;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

pub struct Key<State> {
    cell: Weak<Mutex<Cell<State>>>,
}

impl<State> Key<State> {
    /// Returns the current commit of the state variable if it is still referenced within the most
    /// recent revision of the topology.
    pub fn read(&self) -> Option<Commit<State>> {
        self.cell.upgrade().map(|cell| {
            let mut cell = cell.lock();
            cell.read()
        })
    }

    /// Commits a new value to the state variable if it is still referenced within the most recent
    /// revision. Accepts an updater to which the current value is passed by reference, and which
    /// has the option to skip a full update by returning `None`.
    pub fn update(&self, updater: impl FnOnce(&State) -> Option<State>) {
        if let Some(cell) = self.cell.upgrade() {
            let mut cell = cell.lock();
            cell.commit(updater);
        }
    }
}

impl<State> Key<State>
where
    State: PartialEq,
{
    /// Commits a new state value if it is unequal to the current value and the state variable was
    /// referenced in the last revision.
    pub fn set(&self, new: State) {
        self.update(|prev| if prev == &new { None } else { Some(new) });
    }
}

struct Cell<State> {
    current: Commit<State>,
    last_rooted: Revision,
    pending: Option<Commit<State>>,
    waker: LoopWaker,
}

impl<State> Cell<State> {
    fn read(&mut self) -> Commit<State> {
        if let Some(pending) = self.pending.take() {
            self.current = pending;
        }

        self.current.clone()
    }

    fn commit(&mut self, op: impl FnOnce(&State) -> Option<State>) {
        if let Some(pending) = op(&*self.current) {
            self.pending = Some(Commit {
                inner: Arc::new(pending),
                revision: self.last_rooted,
            });
            self.waker.wake();
        }
    }
}
