use {
    crate::{
        memo::*,
        runtime::{Revision, RunLoopWaker},
    },
    parking_lot::Mutex,
    std::{
        cell::Cell,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        ops::Deref,
        rc::Rc,
        sync::Arc,
    },
    topo::bound,
    tracing::*,
};

/// The current revision of state in this component subtree. Set to the last ran [`Revision`] when
/// state receives commits in the subtree of [`crate::Component`]s called by the `Component` to
/// which this chain corresponds.
///
/// We store a chain of [`RevisionNode`]s so that we can notify parent Components that they may
/// need to run. Without building out a nice system for re-running specific component subtrees
/// on state changes, this is the cleanest way of ensuring we call the path from the root to the
/// `Component` which needs to be called.
#[derive(Clone)]
pub(super) struct RevisionChain(Rc<RevisionNode>);

/// A link in the [`RevisionChain`].
struct RevisionNode {
    current: Cell<u64>,
    parent: std::rc::Weak<Self>,
}

impl RevisionChain {
    pub(super) fn new() -> Self {
        let parent = if let Some(parent_state) = topo::Env::get::<Self>() {
            Rc::downgrade(&parent_state.0)
        } else {
            std::rc::Weak::new()
        };

        RevisionChain(Rc::new(RevisionNode {
            parent,
            current: Cell::new(Revision::current().0),
        }))
    }

    pub(super) fn current(&self) -> u64 {
        self.0.current.get()
    }

    fn increment(&self) -> u64 {
        self.0.current.set(self.current() + 1);
        if let Some(parent) = self.0.parent.upgrade() {
            RevisionChain(parent).increment();
        }
        self.current()
    }
}

impl Debug for RevisionChain {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!("{:?}", self.0.current.get()))
    }
}

impl PartialEq for RevisionChain {
    fn eq(&self, other: &Self) -> bool {
        let self_current = self.0.current.get();
        let other_current = other.0.current.get();
        trace!({ self_current, other_current }, "comparing revision chains");
        self_current.eq(&other_current)
    }
}

/// The underlying container of state variables. Vends copies of the latest [`Commit`] for reads
/// and internally [`Weak`] pointers to these structs are used for updating state with a [`Key`].
struct Var<State> {
    current: Commit<State>,
    point: topo::Id,
    rev_path: RevisionChain,
    last_rooted: Revision,
    pending: Option<Commit<State>>,
    waker: RunLoopWaker,
}

impl<State> Var<State> {
    /// Attach this `Var` to a specific callsite, performing any pending commit and returning the
    /// resulting latest commit.
    fn root(&mut self) -> Commit<State> {
        trace!("rooting state var");
        self.last_rooted = Revision::current();
        self.flush();
        self.peek()
    }

    /// Finishes the pending comment if one exists.
    fn flush(&mut self) {
        if let Some(pending) = self.pending.take() {
            self.current = pending;
        }
    }

    /// Snapshots the current commit.
    fn peek(&self) -> Commit<State> {
        self.current.clone()
    }

    /// Initiate a commit to the state variable. The commit will actually complete asynchronously
    /// when the state variable is next rooted in a topological function, flushing the pending
    /// commit.
    fn enqueue_commit(
        &mut self,
        updater: impl FnOnce(&State) -> Option<State>,
    ) -> Option<Revision> {
        trace!("run updater");
        let pending = updater(&self.pending.as_ref().unwrap_or(&self.current));

        if let Some(pending) = pending {
            trace!("pending commit");
            let current = Revision(self.rev_path.increment());
            self.pending = Some(Commit {
                inner: Arc::new(pending),
                point: self.point,
                revision: current,
            });
            self.waker.wake();
            Some(current)
        } else {
            trace!("skipped commit");
            None
        }
    }
}

// TODO state tests

// TODO: proc macro should allow topo functions to declare `arg` optional with a default to
// which the macro can desugar invocations, so you can pass a init closure only.
// TODO: move arg after initializer

/// Root a state [`Var`] at this callsite, returning an up-to-date [`Commit`] of its value and
/// a unique [`Key`] which can be used to commit new values to the variable.
#[doc(hidden)]
#[bound]
pub fn make_state<Arg, Init, Output>(arg: Arg, initializer: Init) -> Key<Output>
where
    Arg: PartialEq + 'static,
    Output: 'static,
    for<'a> Init: FnOnce(&'a Arg) -> Output,
{
    let current_revision = Revision::current();

    let var: Arc<Mutex<Var<Output>>> = memo!(arg, |a| {
        trace!("init var");
        let waker = topo::Env::expect::<RunLoopWaker>().to_owned();
        let var = Var {
            point: topo::Id::current(),
            last_rooted: current_revision,
            rev_path: topo::Env::expect::<RevisionChain>().to_owned(),
            current: Commit {
                revision: current_revision,
                point: topo::Id::current(),
                inner: Arc::new(initializer(a)),
            },
            pending: None,
            waker,
        };

        Arc::new(Mutex::new(var))
    });

    let commit_at_root = var.lock().root();

    Key {
        commit_at_root,
        var,
    }
}

#[macro_export]
macro_rules! state {
    ($arg:expr, $init:expr) => {
        $crate::make_state!($arg, $init)
    };
    (|| $init:expr) => {
        $crate::make_state!((), |()| $init)
    };
    () => {
        $crate::make_state!((), |()| Default::default())
    };
}

/// A read-only pointer to the value of a state variable *at a particular revision*.
///
/// Reads through a commit are not guaranteed to be the latest value visible to the runloop. Commits
/// should be shared and used within the context of a single [`Revision`], being re-loaded from
/// the state variable on each fresh iteration.
#[derive(Debug, Eq, PartialEq)]
pub struct Commit<State> {
    point: topo::Id,
    revision: Revision,
    inner: Arc<State>,
}

impl<State> Clone for Commit<State> {
    fn clone(&self) -> Self {
        Self {
            revision: self.revision,
            point: self.point,
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

impl<State> Display for Commit<State>
where
    State: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!("{}", self.inner))
    }
}

/// A Key commits new values to a state variable. Keys carry a weak reference to the state variable
/// to prevent cycles, which means that all operations called against them are fallible -- we cannot
/// know before calling a method that the state variable is still live.
pub struct Key<State> {
    commit_at_root: Commit<State>,
    var: Arc<Mutex<Var<State>>>,
}

impl<State> Key<State> {
    /// Returns the current commit of the state variable if it is live.
    pub fn read(&self) -> Commit<State> {
        self.var.lock().peek()
    }

    /// Enqueues a new commit to the state variable if it is still live. Accepts an `updater` to
    /// which the current value is passed by reference. If `updater` returns `None` then no commit
    /// is enqueued and the runloop is not woken.
    ///
    /// Returns the [`Revision`] at which the state variable was last rooted if the variable is
    /// live, otherwise returns `None`.
    pub fn update(&self, updater: impl FnOnce(&State) -> Option<State>) -> Option<Revision> {
        self.var.lock().enqueue_commit(updater)
    }
}

impl<State> Key<State>
where
    State: PartialEq,
{
    /// Commits a new state value if it is unequal to the current value and the state variable is
    /// still live.
    pub fn set(&self, new: State) -> Option<Revision> {
        self.update(|prev| if prev == &new { None } else { Some(new) })
    }
}

impl<State> Clone for Key<State> {
    fn clone(&self) -> Self {
        Self {
            commit_at_root: self.commit_at_root.clone(),
            var: self.var.clone(),
        }
    }
}

impl<State> Deref for Key<State> {
    type Target = State;
    fn deref(&self) -> &Self::Target {
        self.commit_at_root.deref()
    }
}

impl<State> Debug for Key<State>
where
    State: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.commit_at_root.fmt(f)
    }
}

impl<State> Display for Key<State>
where
    State: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.commit_at_root.fmt(f)
    }
}

impl<State> PartialEq for Key<State> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.var, &other.var)
    }
}
