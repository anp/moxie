use {
    crate::{canny_map::CannyMap, surface::Surface},
    froggy::{Pointer, Storage},
    parking_lot::{Mutex, MutexGuard, RwLock},
    rental::rental,
    salsa::Database as SalsaDb,
    std::{
        default::Default,
        sync::{Arc, Weak},
    },
};

pub struct Db {
    inner: Arc<RwLock<InnerDb>>,
}

pub struct InnerDb {
    runtime: salsa::Runtime<InnerDb>,
    singletons: CannyMap,
}

impl Db {
    pub fn new() -> Self {
        // FIXME lol this init order is...smelly
        // but the immediate "fix" on my mind is to use unsafe which is smellier
        Self {
            inner: Arc::new(RwLock::new(InnerDb {
                runtime: Default::default(),
                singletons: Default::default(),
            })),
        }
    }

    pub fn with<T>(&self, f: impl FnOnce(&InnerDb) -> T) -> T {
        let db = self.inner.read();
        f(&*db)
    }

    pub fn with_mut<T>(&self, f: impl FnOnce(&mut InnerDb) -> T) -> T {
        let mut db = self.inner.write();
        f(&mut *db)
    }

    pub async fn run(&self, f: impl FnMut()) {
        unimplemented!()
    }
}

pub trait StateStore {
    fn get_state<S: 'static>(
        &self,
        current: Moniker,
        parent: Moniker,
        init: impl FnOnce() -> S,
    ) -> StateGuard<S>;
}

impl StateStore for InnerDb {
    fn get_state<S: 'static>(
        &self,
        current: Moniker,
        _parent: Moniker,
        init: impl FnOnce() -> S,
    ) -> StateGuard<S> {
        let mut guard: Option<StateGuard<S>> = None;

        self.singletons.alter(|s: Option<Silo<S>>| {
            let silo = s.unwrap_or_default();
            guard = Some(silo.get_or_init(current, init));
            Some(silo)
        });

        // FIXME store parent information here to support context later

        guard.unwrap()
    }
}

#[derive(Debug)]
struct Silo<S> {
    storage: Arc<Mutex<InnerSilo<S>>>,
}

// UNSAFE: needed for rental to be happy with our wrapper type here
unsafe impl<S> stable_deref_trait::StableDeref for Silo<S> {}

impl<S> Silo<S> {
    fn get_or_init(&'static self, id: Moniker, init: impl FnOnce() -> S) -> StateGuard<S> {
        // first
        // StateGuard::new(
        let storage_subsilo = CellGuard::new(self.storage.clone(), |silo| silo.lock());
        let guard = StateGuard::new(Box::new(storage_subsilo), |storage| {
            storage.sync_pending();
            for item in storage.iter() {
                if (*item).id == id {
                    return unimplemented!();
                }
            }

            let state_ptr = storage.create(StorageCell::new(id, init()));

            storage[&state_ptr].lock()
        });

        guard
    }
}

impl<S> std::clone::Clone for Silo<S> {
    fn clone(&self) -> Self {
        Silo {
            storage: Arc::clone(&self.storage),
        }
    }
}

impl<S> std::default::Default for Silo<S> {
    // this cant be derived bc apparently rust always wants to put a S: Default bound on this
    // even though the only child type impls Default regardless of its contained type.
    fn default() -> Silo<S> {
        Silo {
            storage: Arc::new(Mutex::new(InnerSilo(Box::new(Storage::new())))),
        }
    }
}

impl<S> std::ops::Deref for Silo<S> {
    type Target = Mutex<InnerSilo<S>>;
    fn deref(&self) -> &Self::Target {
        &*self.storage
    }
}

#[derive(Debug, Default)]
pub struct InnerSilo<S>(Box<Storage<StorageCell<S>>>);

// FIXME UNSAFE make sure this is correct to implement
unsafe impl<S> stable_deref_trait::StableDeref for InnerSilo<S> {}

impl<S> std::ops::Deref for InnerSilo<S> {
    type Target = Storage<StorageCell<S>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<S> std::ops::DerefMut for InnerSilo<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct StorageCell<S> {
    id: Moniker,
    state: Mutex<S>,
}

impl<S> StorageCell<S> {
    fn new(id: Moniker, state: S) -> Self {
        Self {
            id,
            state: Mutex::new(state),
        }
    }

    fn lock(&self) -> MutexGuard<S> {
        self.state.lock()
    }
}

rental! {mod state_rental {
    use {
        super::{InnerSilo, Silo, StorageCell, Storage},
        parking_lot::{Mutex, MutexGuard},
        stable_deref_trait::StableDeref,
        std::{ops::DerefMut, sync::Arc},
    };

    #[rental(debug, deref_suffix, deref_mut_suffix)]
    pub struct CellGuard<INNER: 'static> {
        silo: Arc<Mutex<INNER>>,
        storage: MutexGuard<'silo, INNER>,
    }

    #[rental(debug, deref_suffix, deref_mut_suffix)]
    pub struct StateGuard<S: 'static> {
        storage: Box<CellGuard<InnerSilo<S>>>,
        state: MutexGuard<'storage, S>,
    }
}}
pub use self::state_rental::{CellGuard, StateGuard};

#[salsa::query_group]
pub trait RenderDatabase<S>: SalsaDb + StateStore {
    fn Surface(&self, parent: Moniker) -> ();
}

impl SalsaDb for InnerDb {
    fn salsa_runtime(&self) -> &salsa::Runtime<InnerDb> {
        &self.runtime
    }
}

salsa::database_storage! {
    pub struct DbStorage for InnerDb {
        impl RenderDatabase {
            fn Surface() for SurfaceQuery;
        }
    }
}

#[macro_export]
macro_rules! state {
    ($db:ident, $parent:ident, $init:expr) => {
        $db.get_state(moniker!(&$parent), $parent, $init)
    };
}

/// Provides restricted access to a value in the state store. Immutable access via `Handle::with`
/// does not cause any recomposition to occur. Owned access via `Handle::set` requires that the
/// closure which receives the state value also returns an owned value of the same type, and after
/// it returns a recomposition will be triggered.
#[derive(Clone)]
pub struct Handle<S> {
    state_ptr: Pointer<Mutex<Option<S>>>,
    db: Weak<RwLock<Db>>,
}

impl<S> Handle<S> {
    // TODO: decide whether we want to do this?
    pub fn with<T>(&self, _f: impl FnOnce(&S) -> T) {
        unimplemented!()
    }

    pub fn set(&self, _f: impl FnOnce(S) -> S) {
        if let Some(_db) = self.db.upgrade() {
            // let state = (*db.write()[&self.state_ptr].lock();
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
