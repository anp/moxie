use {
    chashmap::CHashMap,
    futures::{future::FutureObj, task::Spawn},
    parking_lot::{MappedMutexGuard, Mutex, MutexGuard},
    std::{
        any::{Any, TypeId},
        sync::Arc,
    },
};

type ScopedStateCells = Arc<CHashMap<CallsiteId, StateCell>>;
type StateCell = Arc<(TypeId, Mutex<Box<Any + 'static>>)>;

/// Provides a component with access to the persistent state store and futures executor.
///
/// Because `salsa` does not yet support generic queries, we need a concrete type that can be
/// passed as an argument and tracked within the incremental computation system.
#[derive(Clone, Debug)]
pub struct Scope {
    pub id: ScopeId,
    states: ScopedStateCells,
    spawner: Arc<Mutex<crate::Spawner>>,
}

impl Scope {
    pub(crate) fn new(id: ScopeId, spawner: crate::Spawner) -> Self {
        Self {
            id,
            spawner: Arc::new(Mutex::new(spawner)),
            states: Default::default(),
        }
    }

    #[inline]
    pub fn state<S: 'static + Any>(&self, callsite: CallsiteId, f: impl FnOnce() -> S) -> Guard<S> {
        let id = TypeId::of::<S>();

        let mut cell = None;

        self.states.alter(callsite, |prev| {
            if let Some(p) = prev {
                cell = Some(p);
            } else {
                let initialized = f();
                cell = Some(Arc::new((id, Mutex::new(Box::new(initialized)))))
            }
            cell.clone()
        });

        Guard(RentedGuard::new(cell.unwrap(), |cell| {
            MutexGuard::map(cell.1.lock(), |any| any.downcast_mut().unwrap())
        }))
    }

    pub fn task(&self, _callsite: CallsiteId, fut: FutureObj<'static, ()>) {
        // TODO make this abortable
        self.spawner.lock().spawn_obj(fut).unwrap();
    }
}

// FIXME this needs to include a notion of the hash of all included values
impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Arc::ptr_eq(&self.states, &other.states)
    }
}

impl Eq for Scope {}

rental::rental! {
    pub mod rent_state {
        use super::*;

        /// A `Guard` provides a reference to state in the database. It is returned by
        /// the `state!` macro and can also be used to later enqueue mutations for the state
        /// database.
        #[rental(deref_suffix, deref_mut_suffix)]
        pub struct Guard<S: 'static> {
            cell: StateCell,
            guard: MappedMutexGuard<'cell, S>,
        }
    }
}
use rent_state::Guard as RentedGuard;

pub struct Guard<S: 'static>(RentedGuard<S>);

impl<S: 'static> std::ops::Deref for Guard<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<S: 'static> std::ops::DerefMut for Guard<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

/// A `Handle` provides access to a portion of the state database for use outside of composition.
/// The data pointed to by a `Handle` can be mutated using the `Handle::set` method.
///
/// A `Handle` doesn't directly capture any values from the state database, and instead acquires
/// the appropriate locks when `set` is called.
pub struct Handle<S> {
    __ty_marker: std::marker::PhantomData<S>,
}

/// A `Moniker` represents the coordinates of a code location in the render hierarchy.
///
/// The struct describes a location in the program specific to:
///
/// * a line and column of code,
/// * in a particular element function,
/// * TODO: on a particular round of iteration (straight line code always has a single round),
/// * as well as the moniker which resulted in that particular function's invocation
///
/// It can be derived at any point within any element as long as the parent/invoking/enclosing
/// moniker is available. We guarantee that it's always available in render lifecycle in other ways.
///
/// `Moniker`s are the tool underlying elements, state, context, etc. because they allow us to map
/// from a "pure" function back to a state location.
// TODO: there should probably be an actual Moniker capability that encloses one, right?
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Moniker(usize);

impl Moniker {
    #[doc(hidden)]
    #[inline]
    pub fn new(scope: ScopeId, callsite: &'static str) -> Self {
        Moniker(fxhash::hash(&(scope, callsite)))
    }
}

macro_rules! moniker {
    ($parent:expr) => {
        $crate::prelude::Moniker::new($parent, concat!(file!(), "@", line!(), ":", column!()))
    };
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScopeId(Moniker);

impl ScopeId {
    pub fn new(callsite: Moniker) -> Self {
        Self(callsite)
    }

    pub(crate) fn root() -> Self {
        Self(Moniker(fxhash::hash(&0)))
    }
}

macro_rules! scope {
    () => {
        $crate::prelude::ScopeId::root()
    };
    ($parent:expr) => {
        $crate::prelude::ScopeId::new(moniker!($parent))
    };
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallsiteId {
    site: Moniker,
    scope: ScopeId,
}

impl CallsiteId {
    pub fn new(scope: ScopeId, site: Moniker) -> Self {
        Self { scope, site }
    }
}

macro_rules! callsite {
    ($parent:expr) => {
        $crate::prelude::CallsiteId::new($parent, moniker!($parent))
    };
}
