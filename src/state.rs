use {
    crate::{embed::RunLoopWaker, memo::*},
    parking_lot::Mutex,
    std::{
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        ops::Deref,
        sync::Arc,
    },
};

/// The underlying container of state variables. Vends copies of the latest [`Commit`] for [`Key`]s.
struct Var<State> {
    current: Commit<State>,
    point: topo::Id,
    pending: Option<Commit<State>>,
    waker: RunLoopWaker,
}

impl<State> Var<State> {
    /// Attach this `Var` to its callsite, performing any pending commit and returning the
    /// resulting latest commit.
    fn root(&mut self) -> (topo::Id, Commit<State>) {
        if let Some(pending) = self.pending.take() {
            self.current = pending;
        }
        (self.point, self.current.clone())
    }

    /// Returns a reference to the latest value, pending or committed.
    fn latest(&self) -> &State {
        &self.pending.as_ref().unwrap_or(&self.current)
    }

    /// Initiate a commit to the state variable. The commit will actually complete asynchronously
    /// when the state variable is next rooted in a topological function, flushing the pending
    /// commit.
    fn enqueue_commit(&mut self, state: State) {
        self.pending = Some(Commit {
            inner: Arc::new(state),
            point: self.point,
        });
        self.waker.wake();
    }
}

/// Root a state variable at this callsite, returning a [`Key`] to the state variable.
/// Re-initializes the state variable if the capture argument changes.
#[topo::nested]
#[illicit::from_env(waker: &RunLoopWaker)]
pub fn memo_state<Arg, Init, Output>(arg: Arg, initializer: Init) -> Key<Output>
where
    Arg: PartialEq + 'static,
    Output: 'static,
    for<'a> Init: FnOnce(&'a Arg) -> Output,
{
    let var = memo!(arg, |a| {
        let var = Var {
            point: topo::Id::current(),
            current: Commit {
                point: topo::Id::current(),
                inner: Arc::new(initializer(a)),
            },
            pending: None,
            waker: waker.to_owned(),
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

/// Convenience wrapper around [`memo_state`].
#[macro_export]
macro_rules! state {
    ($arg:expr, $init:expr) => {
        $crate::memo_state!($arg, $init)
    };
    (|| $init:expr) => {
        $crate::memo_state!((), |()| $init)
    };
    () => {
        $crate::memo_state!((), |()| Default::default())
    };
}

/// A read-only pointer to the value of a state variable *at a particular revision*.
///
/// Reads through a commit are not guaranteed to be the latest value visible to the runtime. Commits
/// should be shared and used within the context of a single [`Revision`], being re-loaded from
/// the state variable each time.
#[derive(Debug, Eq, PartialEq)]
struct Commit<State> {
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

/// A `Key` offers access to a state variable. The key allows reads of the state variable through
/// a snapshot taken when the `Key` was created. Writes are supported with [Key::update] and
/// [Key::set].
///
/// They are created with the [`memo_state`] and [`state`] macros.
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

    /// Runs `updater` with a reference to the state variable's latest value, and enqueues a commit
    /// to the variable if `updater` returns `Some`. Returns the [`Revision`] at which the state
    /// variable was last rooted if the variable is live, otherwise returns `None`.
    ///
    /// Enqueuing the commit invokes the state change waker registered with the [Runtime] (if any)
    /// to ensure that the code embedding the runtime schedules another call of [run_once].
    ///
    /// This should be called during event handlers or other code which executes outside of a
    /// `Revision`'s execution, otherwise unpredictable waker behavior may be obtained.
    ///
    /// [Runtime]: crate::embed::Runtime
    /// [run_once]: crate::embed::Runtime::run_once
    pub fn update(&self, updater: impl FnOnce(&State) -> Option<State>) {
        let mut var = self.var.lock();
        if let Some(new) = updater(var.latest()) {
            var.enqueue_commit(new);
        }
    }
}

impl<State> Key<State>
where
    State: PartialEq,
{
    /// Commits a new state value if it is unequal to the current value and the state variable is
    /// still live. Has the same properties as [update](crate::state::Key::update) regarding waking
    /// the runtime.
    pub fn set(&self, new: State) {
        self.update(|prev| if prev == &new { None } else { Some(new) });
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
    /// Keys are considered equal if they point to the same state variable. Importantly, they will
    /// compare as equal even if they contain different snapshots of the state variable due to
    /// having been initialized in different revisions.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.var, &other.var)
    }
}
