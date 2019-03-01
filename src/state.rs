use {
    crate::caps::CallsiteId,
    chashmap::CHashMap,
    parking_lot::{MappedMutexGuard, Mutex, MutexGuard},
    std::{
        any::{Any, TypeId},
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc, Weak,
        },
    },
};

#[derive(Clone, Debug)]
pub(crate) struct States {
    revision: Arc<AtomicU64>,
    contents: Arc<CHashMap<CallsiteId, Arc<StateCell>>>,
}

impl States {
    pub(crate) fn get_or_init<State: 'static>(
        &self,
        callsite: CallsiteId,
        init: impl FnOnce() -> State,
    ) -> Guard<State> {
        let mut cell = None;

        self.contents.alter(callsite, |prev| {
            if let Some(p) = prev {
                cell = Some(p);
            } else {
                // FIXME handle panics in the init fn properly
                let initialized = init();
                cell = Some(Arc::new(StateCell::new(
                    Arc::downgrade(&self.revision),
                    initialized,
                )));
            }
            cell.clone()
        });

        let cell = cell.unwrap();

        Guard {
            cell: cell.downgrade(),
            rented: crate::state::RentedGuard::new(cell, |cell| {
                MutexGuard::map((*cell).0.contents.lock(), |any| {
                    let anon: &mut Box<Any> = any.as_mut().unwrap();

                    let casted: &mut Option<State> = anon.downcast_mut().unwrap();

                    casted.as_mut().unwrap()
                })
            }),
        }
    }
}

impl Default for States {
    fn default() -> Self {
        States {
            revision: Arc::new(AtomicU64::new(0)),
            contents: Arc::new(CHashMap::new()),
        }
    }
}

impl PartialEq for States {
    fn eq(&self, other: &Self) -> bool {
        self.revision.load(Ordering::SeqCst) == other.revision.load(Ordering::SeqCst)
            && Arc::ptr_eq(&self.contents, &other.contents)
    }
}

/// A `Handle` provides access to a portion of the state database for use outside of composition.
/// The data pointed to by a `Handle` can be mutated using the `Handle::set` method.
///
/// A `Handle` doesn't directly capture any values from the state database, and instead acquires
/// the appropriate locks when `set` is called.
pub struct Handle<S> {
    cell: WeakStateCell,
    __ty_marker: std::marker::PhantomData<S>,
}

// FIXME this is a huge safety/soundness hole!
unsafe impl<S> Send for Handle<S> {}

impl<State: 'static> Handle<State> {
    // TODO if the type impls Hash, we should see whether we can skip updating the revision
    pub fn set(&self, updater: impl FnOnce(State) -> State) {
        if let Some(cell) = self.cell.upgrade() {
            // FIXME handle panics in the updater

            let mut inner = cell.0.contents.lock();
            let inner: &mut Option<Box<Any>> = &mut *inner;
            let inner: &mut Option<State> = inner
                .as_mut()
                .map(|anon| {
                    anon.downcast_mut()
                        .expect("failed type cast from state cell")
                })
                .unwrap();

            let prev: State = inner.take().unwrap();

            // FIXME should be panic-safe
            let new = updater(prev);
            inner.replace(new);
            cell.tick_revision();
        }
    }
}

pub struct Guard<S: 'static> {
    pub(crate) cell: WeakStateCell,
    pub(crate) rented: RentedGuard<S>,
}

impl<State: 'static> Guard<State> {
    pub fn handle(&self) -> Handle<State> {
        Handle {
            cell: self.cell.clone(),
            __ty_marker: std::marker::PhantomData,
        }
    }
}

impl<S: 'static> std::ops::Deref for Guard<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &*self.rented
    }
}

impl<S: 'static> std::ops::DerefMut for Guard<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.rented
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StateCell(Arc<StateCellInner>);

impl StateCell {
    fn new<State: 'static>(scope_revision: Weak<AtomicU64>, state: State) -> Self {
        let ty = TypeId::of::<State>();
        let contents: Box<Option<State>> = Box::new(Some(state));
        let contents = Mutex::new(Some(contents as Box<Any>));
        StateCell(Arc::new(StateCellInner {
            ty,
            contents,
            scope_revision,
        }))
    }

    fn downgrade(&self) -> WeakStateCell {
        WeakStateCell(Arc::downgrade(&self.0))
    }

    fn tick_revision(&self) {
        self.0
            .scope_revision
            .upgrade()
            .map(|inner| inner.fetch_add(1, Ordering::SeqCst));
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WeakStateCell(Weak<StateCellInner>);

impl WeakStateCell {
    fn upgrade(&self) -> Option<StateCell> {
        self.0.upgrade().map(StateCell)
    }
}

unsafe impl Send for StateCellInner {}
unsafe impl Sync for StateCellInner {}

#[derive(Debug)]
pub(crate) struct StateCellInner {
    ty: TypeId,
    scope_revision: Weak<AtomicU64>,
    contents: AnonymousState,
}

// TODO make this have a send bound in the default case
type AnonymousState = Mutex<Option<Box<Any + 'static>>>;

// FIXME scope needs to include a revision, and components should be given a handle to the scope,
// there should only be one canonical one at a time, rather than having a bunch of split up handles

rental::rental! {
    mod rent_state {
        use super::*;

        /// A `Guard` provides a reference to state in the database. It is returned by
        /// the `state!` macro and can also be used to later enqueue mutations for the state
        /// database.
        #[rental(deref_suffix, deref_mut_suffix)]
        pub(crate) struct Guard<S: 'static> {
            cell: Arc<StateCell>,
            guard: MappedMutexGuard<'cell, S>,
        }
    }
}
use rent_state::Guard as RentedGuard;
