use {
    froggy::{Pointer, Storage},
    parking_lot::{Mutex, RwLock},
    std::sync::{Arc, Weak},
    {crate::surface::Surface, salsa::Database as SalsaDb},
};

pub struct Db {
    runtime: salsa::Runtime<Db>,
    singletons: RwLock<anymap::AnyMap>,
    weak_self: Option<Weak<RwLock<Self>>>,
}

pub type MoxieDb = Arc<RwLock<Db>>;

#[salsa::query_group]
pub trait RenderDatabase<S>: SalsaDb + SelfReferential + StateStore {
    fn Surface(&self, parent: Moniker, props: ()) -> ();
}

impl Db {
    pub fn new() -> MoxieDb {
        // FIXME lol this init order is...smelly
        // but the immediate "fix" on my mind is to use unsafe which is smellier
        let mut new = Arc::new(RwLock::new(Self {
            runtime: Default::default(),
            singletons: RwLock::new(anymap::AnyMap::new()),
            weak_self: None,
        }));

        let new_weak_self = Arc::downgrade(&new);

        {
            let mut new = new.write();
            new.weak_self = Some(new_weak_self);
        }

        new
    }
}

pub trait StateStore {
    fn state_guard<S>(
        &self,
        current: Moniker,
        parent: Moniker,
        init: impl FnOnce() -> S,
    ) -> Guard<S>;
}

impl StateStore for Db {
    fn state_guard<S>(
        &self,
        current: Moniker,
        parent: Moniker,
        init: impl FnOnce() -> S,
    ) -> Guard<S> {
        unimplemented!()
    }
}

impl SalsaDb for Db {
    fn salsa_runtime(&self) -> &salsa::Runtime<Db> {
        &self.runtime
    }
}

salsa::database_storage! {
    pub struct DbStorage for Db {
        impl RenderDatabase {
            fn Surface() for SurfaceQuery;
        }
    }
}

#[macro_export]
macro_rules! state {
    ($db:ident, $parent:ident, $init:expr) => {
        $db.state_guard(moniker!(&$parent), $parent, $init)
    };
}

/// A `Guard` is the most powerful state "capability" and is returned directly by the `state!`
/// macro. Its lifetime is tied to scope in which the state macro was called. Because it can only
/// be used during composition it provides mutable access to the state value. It should be
/// downgraded to a `Handle` any time access to the state is needed outside of composition (e.g.
/// for event handling, I/O, timers, etc.).
pub struct Guard<S> {
    state_ptr: Pointer<Mutex<Option<S>>>,
    store: MoxieDb,
}

#[derive(Clone)]
pub struct Handle<S> {
    state_ptr: Pointer<Mutex<Option<S>>>,
    db: Weak<Storage<Mutex<Option<S>>>>,
}

impl<S> Handle<S> {
    pub fn set(&self, f: impl FnOnce(S) -> S) {
        if let Some(db) = self.db.upgrade() {
            let state = (*db)[&self.state_ptr].lock();
            unimplemented!()
        }
    }
}

/// A `Moniker` is effectively the coordinates of a code location in the render hierarchy.
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
pub struct Moniker {
    this: usize,
    parent: usize,
}

macro_rules! moniker {
    ($parent:expr) => {
        $crate::prelude::Moniker::new($parent, concat!(file!(), "@", line!(), ":", column!()))
    };
}

impl Moniker {
    #[doc(hidden)]
    pub fn new(parent: &Moniker, callsite: &'static str) -> Self {
        Moniker {
            this: fxhash::hash(&(parent, callsite)),
            parent: parent.this,
        }
    }

    pub(crate) fn root() -> Self {
        moniker!(&Moniker {
            this: fxhash::hash(&0),
            parent: 0
        })
    }
}

pub trait SelfReferential {
    type Returned;
    fn new_weak(&self) -> Weak<Self::Returned>;
    fn new_arc(&self) -> Arc<Self::Returned>;
}

impl SelfReferential for Db {
    type Returned = RwLock<Self>;
    fn new_weak(&self) -> Weak<Self::Returned> {
        self.weak_self.as_ref().unwrap().clone()
    }
    fn new_arc(&self) -> Arc<Self::Returned> {
        self.weak_self.as_ref().unwrap().upgrade().unwrap()
    }
}
