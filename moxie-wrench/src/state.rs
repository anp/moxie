use {
    crate::surface::surface,
    chashmap::CHashMap,
    parking_lot::{MappedMutexGuard, Mutex, MutexGuard},
    salsa::Database as SalsaDb,
    std::{
        any::{Any, TypeId},
        collections::HashMap,
        hash::Hash,
        sync::Arc,
    },
};

/// A `Composer` is the primary entry point to moxie's runtime systems. It contains the salsa
/// incremental storage, various interners, and is passed to every composable function.
#[salsa::database(ComposeStorage)]
#[derive(Default)]
pub struct Composer {
    runtime: salsa::Runtime<Composer>,
    states: CHashMap<ScopeId, ScopeState>,
}

impl Composer {
    pub fn new() -> Self {
        Default::default()
    }
}

#[salsa::query_group(ComposeStorage)]
pub trait ComposeDb: SalsaDb + StateDb {
    fn surface(&self, parent: ScopeId) -> ();

    // TODO find a way to invalidate this query when the state of the corresponding scope changes
    fn state(&self, scope: ScopeId) -> Port;
}

fn state(compose: &impl ComposeDb, scope: ScopeId) -> Port {
    let mut port = None;

    compose.store().alter(scope, |prev: Option<ScopeState>| {
        let state = prev.unwrap_or_default();

        port = Some(Port {
            scope,
            states: state.clone(),
        });

        Some(state)
    });

    port.unwrap()
}

// FIXME this should not be public
pub trait StateDb {
    fn store(&self) -> &CHashMap<ScopeId, ScopeState>;
}

impl StateDb for Composer {
    fn store(&self) -> &CHashMap<ScopeId, ScopeState> {
        &self.states
    }
}

impl SalsaDb for Composer {
    fn salsa_runtime(&self) -> &salsa::Runtime<Composer> {
        &self.runtime
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScopeState(Arc<CHashMap<CallsiteId, StateCell>>);

type StateCell = Arc<(TypeId, Mutex<Box<Any + Send + 'static>>)>;

impl ScopeState {
    fn get<S: 'static + Any + Send>(
        &self,
        callsite: CallsiteId,
        f: impl FnOnce() -> S,
    ) -> Guard<S> {
        let id = TypeId::of::<S>();

        let mut cell = None;

        self.0.alter(callsite, |prev| {
            if let Some(p) = prev {
                cell = Some(p);
            } else {
                let initialized = f();
                cell = Some(Arc::new((id, Mutex::new(Box::new(initialized)))))
            }
            cell.clone()
        });

        Guard::new(cell.unwrap(), |cell| {
            MutexGuard::map(cell.1.lock(), |any| any.downcast_mut().unwrap())
        })
    }
}

impl PartialEq for ScopeState {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ScopeState {}

/// Provides a component with access to the persistent state store.
///
/// Because `salsa` does not yet support generic queries, we need a concrete type that can be
/// passed as an argument and tracked within the incremental computation system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Port {
    scope: ScopeId,
    states: ScopeState,
}

impl Port {
    pub fn get<S: 'static + Any + Send>(
        &self,
        scope: CallsiteId,
        f: impl FnOnce() -> S,
    ) -> Guard<S> {
        self.states.get(scope, f)
    }
}

rental::rental! {
    pub mod rent_state {
        use super::*;

        /// A `Guard` provides an immutable reference to state in the database. It is returned by
        /// the `state!` macro and can also be used to later enqueue mutations for the state
        /// database.
        #[rental(deref_suffix)]
        pub struct Guard<S: 'static> {
            cell: StateCell,
            guard: MappedMutexGuard<'cell, S>,
        }
    }
}
pub use rent_state::Guard;

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
