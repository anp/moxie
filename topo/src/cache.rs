use std::{
    any::{Any, TypeId},
    cmp::Eq,
    collections::HashMap,
    hash::Hash,
};

/// A `Cache` holds results from arbitrary queries for later retrieval. Each
/// query is indexed by a "scope" and the type of the query's inputs and
/// outputs. When collecting garbage, values are retained if they were
/// referenced since the last GC.
#[derive(Debug)]
pub struct Cache<Scope> {
    inner: HashMap<Scope, Namespace>,
}
type Namespace = HashMap<Query, (Liveness, Box<dyn Any>)>;

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
    pub fn get<Input, Output>(&mut self, scope: &Scope, input: &Input) -> Option<&Output>
    where
        Input: PartialEq + 'static,
        Output: 'static,
    {
        let namespace = self.inner.get_mut(scope)?;
        let query = Query::get::<Input, Output>();
        let (ref mut liveness, erased) = namespace.get_mut(&query)?;
        let (ref stored_input, ref stored): &(Input, Output) = erased.downcast_ref().unwrap();
        if stored_input == input {
            *liveness = Liveness::Live;
            Some(stored)
        } else {
            None
        }
    }

    /// Store the result of a query. It will not be GC'd this revision.
    pub fn store<Input, Output>(&mut self, scope: Scope, input: Input, output: Output)
    where
        Input: 'static,
        Output: 'static,
    {
        let (query, erased) = Query::insert(input, output);
        self.inner.entry(scope).or_default().insert(query, (Liveness::Live, erased));
    }

    /// Drops memoized values that were not referenced since the last call
    /// and sets all remaining values to be dropped by default in the next call.
    pub fn gc(&mut self) {
        for namespace in self.inner.values_mut() {
            namespace.retain(|_, (liveness, _)| liveness == &Liveness::Live);
            namespace.values_mut().for_each(|(liveness, _)| *liveness = Liveness::Dead);
        }

        self.inner.retain(|_, namespace| !namespace.is_empty());
    }
}

/// Each query has an `Input`, and an `Output` which together can be
/// thought of as defining a function: `(input) -> output`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
struct Query {
    /// The type of input the query accepts.
    input: TypeId,
    /// The type of output the query returns.
    output: TypeId,
}

impl Query {
    fn get<Input, Output>() -> Self
    where
        Input: 'static,
        Output: 'static,
    {
        Self { input: TypeId::of::<Input>(), output: TypeId::of::<Output>() }
    }

    fn insert<Input, Output>(input: Input, output: Output) -> (Self, Box<dyn Any>)
    where
        Input: 'static,
        Output: 'static,
    {
        (Query::get::<Input, Output>(), Box::new((input, output)) as Box<dyn Any>)
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
