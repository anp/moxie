//! Moxie implements "topological memoization" with storage in its runtime.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    cmp::Eq,
    collections::HashMap,
    hash::Hash,
    rc::Rc,
};

/// A shared pointer to the memoization storage singleton for a given runtime.
#[derive(Clone, Debug)]
pub(crate) struct MemoStore<Id = topo::Id>(Rc<RefCell<MemoStorage<Id>>>);

impl<Id> Default for MemoStore<Id> {
    fn default() -> Self {
        MemoStore(Rc::new(RefCell::new(MemoStorage::default())))
    }
}

impl<Id> MemoStore<Id>
where
    Id: Clone + Eq + Hash,
{
    /// Provides a closure-based memoization API on top of [`MemoStorage`]'s
    /// mutable API. Steps:
    ///
    /// 1. if matching cached value, mark live and return
    /// 2. no cached value, initialize new one
    /// 3. store new value as live, return
    ///
    /// Both (1) and (3) require mutable access to storage. We want to allow
    /// nested memoization eventually so it's important that (2) *doesn't*
    /// use mutable access to storage.
    pub fn memo_with<Arg, Stored, Ret>(
        &self,
        id: Id,
        arg: Arg,
        init: impl FnOnce(&Arg) -> Stored,
        with: impl FnOnce(&Stored) -> Ret,
    ) -> Ret
    where
        Arg: PartialEq + 'static,
        Stored: 'static,
        Ret: 'static,
    {
        if let Some(stored) = { self.0.borrow_mut().get_if_arg_eq(id.clone(), &arg) } {
            return with(stored);
        }

        let to_store = init(&arg);
        let to_return = with(&to_store);
        self.0.borrow_mut().insert(id, arg, to_store);
        to_return
    }

    /// Drops memoized values that were not referenced during the last
    /// `Revision`.
    pub fn gc(&self) {
        self.0.borrow_mut().gc();
    }
}

/// The memoization storage for a `Runtime`. Stores memoized values by a
/// `MemoIndex`, exposing a garbage collection API to the embedding `Runtime`.
#[derive(Debug)]
pub(crate) struct MemoStorage<Id> {
    memos: HashMap<MemoIndex<Id>, (Liveness, Box<dyn Any>)>,
}
type MemoIndex<Id> = (Id, TypeId, TypeId);

impl<Id> Default for MemoStorage<Id> {
    fn default() -> Self {
        MemoStorage { memos: HashMap::new() }
    }
}

impl<Id> MemoStorage<Id>
where
    Id: Eq + Hash,
{
    /// Return a reference to the stored value if `arg` equals the
    /// previously-stored argument. If a reference is returned the storage
    /// is marked [`Liveness::Live`] and will not be GC'd at the end of the
    /// current [`crate::embed::Revision`].
    fn get_if_arg_eq<Arg, Stored>(&mut self, id: Id, arg: &Arg) -> Option<&Stored>
    where
        Arg: PartialEq + 'static,
        Stored: 'static,
    {
        let key = (id, TypeId::of::<Arg>(), TypeId::of::<Stored>());
        let (ref mut liveness, erased) = self.memos.get_mut(&key)?;
        let (ref stored_arg, ref stored): &(Arg, Stored) = erased.downcast_ref().unwrap();
        if stored_arg == arg {
            *liveness = Liveness::Live;
            Some(stored)
        } else {
            None
        }
    }

    /// Store the new value. It will be `Live` for the current revision.
    fn insert<Arg, Stored>(&mut self, id: Id, arg: Arg, to_store: Stored)
    where
        Arg: 'static,
        Stored: 'static,
    {
        let key = (id, TypeId::of::<Arg>(), TypeId::of::<Stored>());
        let erased = Box::new((arg, to_store)) as Box<dyn Any>;
        self.memos.insert(key, (Liveness::Live, erased));
    }

    /// Drops memoized values that were not referenced during the last revision,
    /// removing all `Dead` storage values and sets all remaining values to
    /// `Dead` for the next GC execution.
    fn gc(&mut self) {
        self.memos.retain(|_, (liveness, _)| liveness == &Liveness::Live);
        self.memos.values_mut().for_each(|(liveness, _)| *liveness = Liveness::Dead);
    }
}

/// Describes the outcome for a memoization value if a garbage collection were
/// to occur when observed. During the run of a `Revision` any memoized values
/// which are initialized or read are marked as `Live`. At the end of a
/// `Revision`,
#[derive(Debug, PartialEq)]
enum Liveness {
    /// The memoized value would be retained in a GC right now.
    Live,
    /// The memoized value would be dropped in a GC right now.
    Dead,
}
