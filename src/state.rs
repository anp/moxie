use {
    crate::{memo::*, Revision, RunLoopWaker},
    parking_lot::Mutex,
    std::{
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        ops::Deref,
        sync::Arc,
    },
    tracing::*,
};

/// The underlying container of state variables. Vends copies of the latest [`Commit`] for reads
/// and internally [`Weak`] pointers to these structs are used for updating state with a [`Key`].
struct Var<State> {
    current: Commit<State>,
    point: topo::Id,
    pending: Option<Commit<State>>,
    waker: RunLoopWaker,
}

impl<State> Var<State> {
    /// Attach this `Var` to a specific callsite, performing any pending commit and returning the
    /// resulting latest commit.
    fn root(&mut self) -> (topo::Id, Commit<State>) {
        self.flush();
        (self.point, self.peek())
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

    fn latest(&self) -> &State {
        &self.pending.as_ref().unwrap_or(&self.current)
    }

    /// Initiate a commit to the state variable. The commit will actually complete asynchronously
    /// when the state variable is next rooted in a topological function, flushing the pending
    /// commit.
    fn enqueue_commit(&mut self, state: State) -> Option<Revision> {
        let current = Revision::current();
        self.pending = Some(Commit {
            inner: Arc::new(state),
            point: self.point,
        });
        self.waker.wake();
        Some(current)
    }
}

// TODO state tests

// TODO: proc macro should allow topo functions to declare `arg` optional with a default to
// which the macro can desugar invocations, so you can pass a init closure only.
// TODO: move arg after initializer

/// Root a state [`Var`] at this callsite, returning an up-to-date [`Commit`] of its value and
/// a unique [`Key`] which can be used to commit new values to the variable.
#[topo::aware]
pub fn make_state<Arg, Init, Output>(arg: Arg, initializer: Init) -> Key<Output>
where
    Arg: PartialEq + 'static,
    Output: 'static,
    for<'a> Init: FnOnce(&'a Arg) -> Output,
{
    let var: Arc<Mutex<Var<Output>>> = memo!(arg, |a| {
        trace!("init var");
        let waker = topo::Env::expect::<RunLoopWaker>().to_owned();
        let var = Var {
            point: topo::Id::current(),
            current: Commit {
                point: topo::Id::current(),
                inner: Arc::new(initializer(a)),
            },
            pending: None,
            waker,
        };

        Arc::new(Mutex::new(var))
    });

    let (id, commit_at_root) = var.lock().root();

    Key {
        id,
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
    inner: Arc<State>,
}

impl<State> Clone for Commit<State> {
    fn clone(&self) -> Self {
        Self {
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
    id: topo::Id,
    commit_at_root: Commit<State>,
    var: Arc<Mutex<Var<State>>>,
}

impl<State> Key<State> {
    /// Returns the `topo::Id` at which the state variable is bound.
    pub fn id(&self) -> topo::Id {
        self.id
    }

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
        let mut var = self.var.lock();
        updater(var.latest()).and_then(|p| var.enqueue_commit(p))
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
            id: self.id,
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
