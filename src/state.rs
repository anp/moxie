use {
    crate::{memo::*, Revision, RunLoopWaker},
    parking_lot::Mutex,
    std::{
        cell::Cell,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        ops::Deref,
        sync::Arc,
    },
    topo::bound,
    tracing::*,
};

/// The underlying container of state variables. Vends copies of the latest [`Commit`] for reads
/// and internally [`Weak`] pointers to these structs are used for updating state with a [`Key`].
struct Var<State> {
    current: Commit<State>,
    point: topo::Id,
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
            let current = Revision::current();
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
