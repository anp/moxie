use {
    crate::{component::Revision, prelude::*},
    chashmap::CHashMap,
    parking_lot::{MappedMutexGuard, Mutex},
    std::{
        any::{Any, TypeId},
        sync::{Arc, Weak},
    },
};

pub(crate) type States = Arc<CHashMap<CallsiteId, Arc<StateCell>>>;

#[derive(Clone, Debug)]
pub(crate) struct StateCell(pub(crate) Arc<StateCellInner>);

impl StateCell {
    pub(crate) fn new<State: 'static>(scope_revision: Weak<Revision>, state: State) -> Self {
        let ty = TypeId::of::<State>();
        let contents = Mutex::new(Box::new(state) as Box<Any>);
        StateCell(Arc::new(StateCellInner {
            ty,
            contents,
            scope_revision,
        }))
    }

    pub(crate) fn downgrade(&self) -> WeakStateCell {
        WeakStateCell(Arc::downgrade(&self.0))
    }
}

pub(crate) struct WeakStateCell(Weak<StateCellInner>);

impl WeakStateCell {
    fn upgrade(&self) -> Option<StateCell> {
        self.0.upgrade().map(StateCell)
    }
}

#[derive(Debug)]
pub(crate) struct StateCellInner {
    pub(crate) scope_revision: Weak<Revision>,
    pub(crate) ty: TypeId,
    pub(crate) contents: Mutex<Box<Any + 'static>>,
}

// FIXME scope needs to include a revision, and components should be given a handle to the scope,
// there should only be one canonical one at a time, rather than having a bunch of split up handles

rental::rental! {
    pub mod rent_state {
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
pub(crate) use rent_state::Guard as RentedGuard;

pub struct Guard<S: 'static> {
    pub(crate) cell: WeakStateCell,
    pub(crate) rented: RentedGuard<S>,
}

impl<State: 'static> Guard<State> {
    pub fn handle(&self) -> Handle<State> {
        Handle {
            cell: self.cell.upgrade().unwrap(),
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

/// A `Handle` provides access to a portion of the state database for use outside of composition.
/// The data pointed to by a `Handle` can be mutated using the `Handle::set` method.
///
/// A `Handle` doesn't directly capture any values from the state database, and instead acquires
/// the appropriate locks when `set` is called.
pub struct Handle<S> {
    cell: StateCell,
    __ty_marker: std::marker::PhantomData<S>,
}

impl<State> Handle<State> {
    pub fn set(&self, updater: impl FnOnce(State) -> State) {
        unimplemented!()
    }
}

impl<State> std::ops::Deref for Handle<State> {
    type Target = State;
    fn deref(&self) -> &Self::Target {
        unimplemented!()
    }
}
