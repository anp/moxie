//! Moxie implements "topological memoization" with storage in its runtime.

use std::{cell::RefCell, cmp::Eq, hash::Hash, rc::Rc};
use topo::Cache;

/// A shared pointer to the memoization storage singleton for a given runtime.
#[derive(Clone, Debug)]
pub(crate) struct LocalCache<Id = topo::Id>(Rc<RefCell<Cache<Id>>>);

impl<Id> Default for LocalCache<Id> {
    fn default() -> Self {
        LocalCache(Rc::new(RefCell::new(Cache::default())))
    }
}

impl<Id> LocalCache<Id>
where
    Id: Clone + Eq + Hash,
{
    /// Provides a closure-based memoization API on top of [`Cache`]'s
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
        if let Some(stored) = { self.0.borrow_mut().get(id.clone(), &arg) } {
            return with(stored);
        }

        let to_store = init(&arg);
        let to_return = with(&to_store);
        self.0.borrow_mut().store(id, arg, to_store);
        to_return
    }

    /// Drops memoized values that were not referenced during the last
    /// revision.
    pub fn gc(&self) {
        self.0.borrow_mut().gc();
    }
}
