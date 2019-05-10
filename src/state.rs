use {
    crate::{caps::CallsiteId, scope::WeakScope},
    chashmap::CHashMap,
    parking_lot::{MappedMutexGuard, Mutex, MutexGuard},
    std::{
        any::{Any, TypeId},
        panic::UnwindSafe,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc, Weak,
        },
    },
};

#[derive(Clone, Debug, Default)]
pub(crate) struct States {
    revision: Arc<AtomicU64>,
    contents: Arc<CHashMap<CallsiteId, Arc<StateCell>>>,
}

impl States {
    pub(crate) fn get_or_init<State: UnwindSafe + 'static>(
        &self,
        scope: WeakScope,
        callsite: CallsiteId,
        init: impl FnOnce() -> State,
    ) -> Guard<State> {
        let mut cell = None;

        self.contents.alter(callsite, |prev| {
            if let Some(p) = prev {
                cell = Some(p);
            } else {
                let initialized = init();
                cell = Some(Arc::new(StateCell::new(scope.clone(), initialized)));
            }
            cell.clone()
        });

        cell.unwrap().guard()
    }
}

impl PartialEq for States {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.contents, &other.contents)
            && self.revision.load(Ordering::SeqCst) == other.revision.load(Ordering::SeqCst)
    }
}

#[derive(Clone, Debug)]
pub struct Key<S> {
    cell: WeakStateCell,
    __ty_marker: std::marker::PhantomData<S>,
}

impl<State: UnwindSafe + 'static> Key<State> {
    pub fn set(&self, updater: impl FnOnce(State) -> State) {
        if let Some(cell) = self.cell.upgrade() {
            let mut inner = cell.0.lock();
            let prev: State = inner.take().unwrap();
            let new = updater(prev);
            inner.replace(new);
            cell.0.scope.tick();
        }
    }
}

impl<State> UnwindSafe for Key<State> where State: UnwindSafe {}

pub struct Guard<S: 'static> {
    pub(crate) cell: WeakStateCell,
    pub(crate) rented: RentedGuard<S>,
}

impl<State: 'static> Guard<State> {
    pub fn handle(&self) -> Key<State> {
        Key {
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
    fn new<State: UnwindSafe + 'static>(scope: WeakScope, state: State) -> Self {
        let ty = TypeId::of::<State>();
        let contents: Box<Option<State>> = Box::new(Some(state));
        let contents = Mutex::new(Some(contents as Box<Any + UnwindSafe>));
        StateCell(Arc::new(StateCellInner {
            ty,
            contents,
            scope,
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

impl PartialEq for WeakStateCell {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for WeakStateCell {}

pub(crate) struct StateCellInner {
    ty: TypeId,
    scope: WeakScope,
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
