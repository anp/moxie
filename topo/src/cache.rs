use downcast_rs::{impl_downcast, Downcast};
use std::{
    any::{type_name, TypeId},
    cmp::Eq,
    collections::HashMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::Hash,
};

macro_rules! define_cache {
    ($name:ident $(: $bound:ident)?) => {
/// Holds results from arbitrary queries for later retrieval. Each query is indexed
/// by the type and value of a "scope" and the type of the query's inputs and outputs.
///
/// When collecting garbage, values are retained if they were referenced since the last GC.
#[derive(Debug, Default)]
pub struct $name {
    inner: HashMap<Query, Box<dyn Gc $(+ $bound)?>>,
}

impl $name {
    /// Return a reference to the stored output if `input` equals the
    /// previously-stored input. If a reference is returned, the storage
    /// is marked [`Liveness::Live`] and will not be GC'd this revision.
    pub fn get<Scope, Input, Output>(&mut self, scope: &Scope, input: &Input) -> Option<&Output>
    where
        Scope: 'static + Eq + Hash $(+ $bound)?,
        Input: 'static + PartialEq $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        self.get_namespace_mut::<Scope, Input, Output>().get_if_input_eq(scope, input)
    }

    /// Store the result of a query. It will not be GC'd this revision.
    pub fn store<Scope, Input, Output>(&mut self, scope: Scope, input: Input, output: Output)
    where
        Scope: 'static + Eq + Hash $(+ $bound)?,
        Input: 'static + PartialEq $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        self.get_namespace_mut().insert(scope, input, output);
    }

    fn get_namespace_mut<Scope, Input, Output>(&mut self) -> &mut Namespace<Scope, Input, Output>
    where
        Scope: 'static + Eq + Hash $(+ $bound)?,
        Input: 'static + PartialEq $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        let gc: &mut (dyn Gc $(+ $bound)?) = &mut **self
            .inner
            .entry(Query::get::<Scope, Input, Output>())
            .or_insert_with(|| Box::new(Namespace::<Scope, Input, Output>::default()));
        gc.as_any_mut().downcast_mut().unwrap()
    }

    /// Drops memoized values that were not referenced since the last call
    /// and sets all remaining values to be dropped by default in the next call.
    pub fn gc(&mut self) {
        for namespace in self.inner.values_mut() {
            namespace.gc();
        }
    }
}
    };
}

define_cache!(LocalCache);
define_cache!(Cache: Send);

struct Namespace<Scope, Input, Output> {
    inner: HashMap<Scope, (Liveness, Input, Output)>,
}

impl<Scope, Input, Output> Namespace<Scope, Input, Output>
where
    Scope: Eq + Hash + 'static,
    Input: PartialEq + 'static,
    Output: 'static,
{
    fn get_if_input_eq(&mut self, scope: &Scope, input: &Input) -> Option<&Output> {
        let (ref mut liveness, ref stored_input, ref stored) = self.inner.get_mut(scope)?;
        if stored_input == input {
            *liveness = Liveness::Live;
            Some(stored)
        } else {
            None
        }
    }

    fn insert(&mut self, scope: Scope, input: Input, output: Output) {
        self.inner.insert(scope, (Liveness::Live, input, output));
    }
}

impl<Scope, Input, Output> Gc for Namespace<Scope, Input, Output>
where
    Scope: Eq + Hash + 'static,
    Input: 'static,
    Output: 'static,
{
    fn gc(&mut self) {
        self.inner.retain(|_, (l, _, _)| *l == Liveness::Live);
        self.inner.values_mut().for_each(|(l, _, _)| *l = Liveness::Dead);
    }
}

impl<Scope, Input, Output> Default for Namespace<Scope, Input, Output>
where
    Scope: Eq + Hash + 'static,
    Input: 'static,
    Output: 'static,
{
    fn default() -> Self {
        Self { inner: Default::default() }
    }
}

impl<Scope, Input, Output> Debug for Namespace<Scope, Input, Output>
where
    Scope: Eq + Hash + 'static,
    Input: 'static,
    Output: 'static,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_map()
            .entry(&"scope", &type_name::<Scope>())
            .entry(&"input", &type_name::<Input>())
            .entry(&"output", &type_name::<Output>())
            .finish()
    }
}

/// Each query has an `Input`, and an `Output` which together can be
/// thought of as defining a function: `(input) -> output`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
struct Query {
    /// The type of scope by which the query is namespaced.
    scope: TypeId,
    /// The type of input the query accepts.
    input: TypeId,
    /// The type of output the query returns.
    output: TypeId,
}

impl Query {
    fn get<Scope, Input, Output>() -> Self
    where
        Scope: 'static,
        Input: 'static,
        Output: 'static,
    {
        Self {
            scope: TypeId::of::<Scope>(),
            input: TypeId::of::<Input>(),
            output: TypeId::of::<Output>(),
        }
    }
}

/// A type which can contain values of varying liveness.
trait Gc: Downcast + Debug {
    /// Remove dead entries.
    fn gc(&mut self);
}

impl_downcast!(Gc);

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
