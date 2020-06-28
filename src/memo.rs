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

/// A `Cache` holds results from arbitrary queries for later retrieval. Each
/// query is indexed by a "scope" and the type of the query's inputs and
/// outputs. When collecting garbage, values are retained if they were
/// referenced since the last GC.
#[derive(Debug)]
pub struct Cache<Scope> {
    inner: HashMap<Query<Scope>, (Liveness, Box<dyn Any>)>,
}

impl<Scope> Default for Cache<Scope> {
    fn default() -> Self {
        Cache { inner: HashMap::new() }
    }
}

impl<Scope> Cache<Scope>
where
    Scope: Eq + Hash,
{
    /// Return a reference to the stored output if `input` equals the
    /// previously-stored input. If a reference is returned, the storage
    /// is marked [`Liveness::Live`] and will not be GC'd this revision.
    fn get<Input, Output>(&mut self, scope: Scope, input: &Input) -> Option<&Output>
    where
        Input: PartialEq + 'static,
        Output: 'static,
    {
        let query = Query::new::<Input, Output>(scope);
        let (ref mut liveness, erased) = self.inner.get_mut(&query)?;
        let (ref stored_input, ref stored): &(Input, Output) = erased.downcast_ref().unwrap();
        if stored_input == input {
            *liveness = Liveness::Live;
            Some(stored)
        } else {
            None
        }
    }

    /// Store the result of a query. It will not be GC'd this revision.
    fn store<Input, Output>(&mut self, scope: Scope, input: Input, output: Output)
    where
        Input: 'static,
        Output: 'static,
    {
        let query = Query::new::<Input, Output>(scope);
        let erased = Box::new((input, output)) as Box<dyn Any>;
        self.inner.insert(query, (Liveness::Live, erased));
    }

    /// Drops memoized values that were not referenced since the last call
    /// and sets all remaining values to be dropped by default in the next call.
    fn gc(&mut self) {
        self.inner.retain(|_, (liveness, _)| liveness == &Liveness::Live);
        self.inner.values_mut().for_each(|(liveness, _)| *liveness = Liveness::Dead);
    }
}

/// Each query has a `Scope`, an `Input`, and an `Output` which together can be
/// thought of as defining a function: `scope(input) -> output`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
struct Query<Scope> {
    /// The query's scope or its namespace.
    scope: Scope,
    /// The type of input the query accepts.
    input: TypeId,
    /// The type of output the query returns.
    output: TypeId,
}

impl<Scope> Query<Scope> {
    fn new<Input, Output>(scope: Scope) -> Self
    where
        Input: 'static,
        Output: 'static,
    {
        Self { scope, input: TypeId::of::<Input>(), output: TypeId::of::<Output>() }
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
