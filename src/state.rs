use {
    crate::caps::CallsiteId,
    chashmap::CHashMap,
    parking_lot::{MappedMutexGuard, Mutex, MutexGuard},
    std::{
        any::{Any, TypeId},
        hash::{Hash, Hasher},
        panic::UnwindSafe,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc, Weak,
        },
        task::Waker,
    },
};

#[derive(Clone, Debug)]
pub(crate) struct States {
    revision: Arc<AtomicU64>,
    compose_waker: Waker,
    contents: Arc<CHashMap<CallsiteId, Arc<StateCell>>>,
}

impl States {
    pub(crate) fn new(compose_waker: Waker) -> Self {
        Self {
            revision: Arc::new(AtomicU64::new(0)),
            contents: Arc::new(CHashMap::default()),
            compose_waker,
        }
    }

    pub(crate) fn get_or_init<State: UnwindSafe + 'static>(
        &self,
        callsite: CallsiteId,
        init: impl FnOnce() -> State,
    ) -> Guard<State> {
        let mut cell = None;

        self.contents.alter(callsite, |prev| {
            if let Some(p) = prev {
                cell = Some(p);
            } else {
                let initialized = init();
                cell = Some(Arc::new(StateCell::new(
                    Arc::downgrade(&self.revision),
                    self.compose_waker.clone(),
                    initialized,
                )));
            }
            cell.clone()
        });

        cell.unwrap().guard()
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
#[derive(Clone, Debug)]
pub struct Handle<S> {
    cell: WeakStateCell,
    __ty_marker: std::marker::PhantomData<S>,
}

impl<State: UnwindSafe + 'static> Handle<State> {
    pub fn set(&self, updater: impl FnOnce(State) -> State) {
        if let Some(cell) = self.cell.upgrade() {
            let mut inner = cell.0.lock();
            let prev: State = inner.take().unwrap();
            let new = updater(prev);
            inner.replace(new);
        }
    }
}

impl<State> UnwindSafe for Handle<State> where State: UnwindSafe {}

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
    fn new<State: UnwindSafe + 'static>(
        scope_revision: Weak<AtomicU64>,
        compose_waker: Waker,
        state: State,
    ) -> Self {
        let ty = TypeId::of::<State>();
        let contents: Box<Option<State>> = Box::new(Some(state));
        let contents = Mutex::new(Some(contents as Box<Any + UnwindSafe>));
        StateCell(Arc::new(StateCellInner {
            ty,
            contents,
            compose_waker,
            scope_revision,
        }))
    }

    fn guard<State: UnwindSafe + 'static>(&self) -> Guard<State> {
        Guard {
            cell: self.downgrade(),
            rented: crate::state::RentedGuard::new(Arc::new(self.clone()), |cell| {
                MappedMutexGuard::map(cell.0.lock(), |opt| opt.as_mut().unwrap())
            }),
        }
    }

    fn downgrade(&self) -> WeakStateCell {
        WeakStateCell(Arc::downgrade(&self.0))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WeakStateCell(Weak<StateCellInner>);

impl WeakStateCell {
    fn upgrade(&self) -> Option<StateCell> {
        self.0.upgrade().map(StateCell)
    }
}

impl Hash for WeakStateCell {
    fn hash<H: Hasher>(&self, _h: &mut H) {
        unimplemented!()
    }
}

impl PartialEq for WeakStateCell {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for WeakStateCell {}

pub(crate) struct StateCellInner {
    ty: TypeId,
    scope_revision: Weak<AtomicU64>,
    compose_waker: Waker,
    contents: Mutex<Option<Box<Any + UnwindSafe + 'static>>>,
}

impl StateCellInner {
    fn lock<State: UnwindSafe + 'static>(&self) -> MappedMutexGuard<Option<State>> {
        assert_eq!(TypeId::of::<State>(), self.ty);

        MutexGuard::map(
            self.contents.lock(),
            |any: &mut Option<Box<_>>| -> &mut Option<State> {
                let anon: &mut Box<Any + UnwindSafe> = any.as_mut().unwrap();
                // UNSAFE(anp): we need to strip the `UnwindSafe` bound from this box to downcast.
                //              luckily for us, the value we're attempting to deref to impls it too.
                let anon: &mut Box<Any> =
                    unsafe { &mut *(anon as *mut Box<Any + UnwindSafe> as *mut Box<Any>) };
                let casted: &mut Option<State> = anon.downcast_mut().unwrap();
                casted
            },
        )
    }

    // FIXME(anp): add a dropguard here probably?
    fn tick_revision(&self) {
        self.scope_revision
            .upgrade()
            .map(|inner| inner.fetch_add(1, Ordering::SeqCst));
        self.compose_waker.clone().wake();
    }
}

impl std::fmt::Debug for StateCellInner {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

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
