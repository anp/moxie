use downcast_rs::{impl_downcast, Downcast};
use parking_lot::Mutex;
use std::{
    any::{type_name, TypeId},
    borrow::Borrow,
    cell::RefCell,
    cmp::Eq,
    collections::HashMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::Hash,
    rc::Rc,
    sync::Arc,
};

macro_rules! define_cache {
    ($name:ident $(: $bound:ident)?, $($rest:tt)*) => {
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
    /// is marked live and will not be GC'd this revision.
    pub fn get<Query, Scope, Arg, Input, Output>(&mut self, query: &Query, input: &Arg) -> Option<&Output>
    where
        Query: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: 'static + Borrow<Query> + Eq + Hash $(+ $bound)?,
        Arg: PartialEq<Input> + ?Sized,
        Input: 'static + Borrow<Arg> $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        self.get_namespace_mut::<Scope, Input, Output>().get_if_input_eq::<Query, Arg>(query, input)
    }

    /// Store the result of a query. It will not be GC'd this revision.
    pub fn store<Scope, Input, Output>(&mut self, scope: Scope, input: Input, output: Output)
    where
        Scope: 'static + Eq + Hash $(+ $bound)?,
        Input: 'static $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        self.get_namespace_mut().insert(scope, input, output);
    }

    fn get_namespace_mut<Scope, Input, Output>(&mut self) -> &mut Namespace<Scope, Input, Output>
    where
        Scope: 'static + Eq + Hash $(+ $bound)?,
        Input: 'static $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        let gc: &mut (dyn Gc $(+ $bound)?) = &mut **self
            .inner
            .entry(Query::get::<Scope, Input, Output>())
            .or_insert_with(|| Box::new(Namespace::<Scope, Input, Output>::default()));
        gc.as_any_mut().downcast_mut().unwrap()
    }

    /// Drops cached values that were not referenced since the last call
    /// and sets all remaining values to be dropped by default in the next call.
    pub fn gc(&mut self) {
        for namespace in self.inner.values_mut() {
            namespace.gc();
        }
    }
}

paste::item! {
    define_cache! {
        @handle $name $(: $bound)?, [<$name Handle>], $($rest)*
    }
}
    };
    (
        @handle
        $name:ident $(: $bound:ident)?,
        $handle:ident,
        $shared:ident,
        $lock:ident :: $acquire:ident
    ) => {

/// Provides access to a shared cache which stores results from arbitrary queries
/// for later retrieval.
#[derive(Clone)]
pub struct $handle {
    inner: $shared<$lock<$name>>,
}

impl Default for $handle {
    fn default() -> Self {
        Self {
            inner: $shared::new($lock::new($name::default()))
        }
    }
}

impl $handle {
    /// Provides a closure-based caching API on top of the underlying
    /// mutable API. Steps:
    ///
    /// 1. if matching cached value, mark live and return
    /// 2. no cached value, initialize new one
    /// 3. store new value as live, return
    ///
    /// Both (1) and (3) require mutable access to storage. We want to allow
    /// nested cached `init`s eventually so it's important that (2) *doesn't*
    /// use mutable access to storage.
    pub fn cache_with<Scope, Arg, Input, Output, Ret>(
        &self,
        scope: Scope,
        arg: &Arg,
        init: impl FnOnce(&Input) -> Output,
        with: impl FnOnce(&Output) -> Ret,
    ) -> Ret
    where
        Scope: 'static + Clone + Eq + Hash $(+ $bound)?,
        Arg: PartialEq<Input> + ToOwned<Owned=Input> + ?Sized,
        Input: 'static + Borrow<Arg> $(+ $bound)?,
        Output: 'static $(+ $bound)?,
        Ret: 'static $(+ $bound)?,
    {
        if let Some(stored) = { self.inner.$acquire().get(&scope, arg) } {
            return with(stored);
        }

        let arg = arg.to_owned();
        let to_store = init(&arg);
        let to_return = with(&to_store);
        self.inner.$acquire().store(scope, arg, to_store);
        to_return
    }

    /// See `gc` on the inner cache type.
    pub fn gc(&self) {
        self.inner.$acquire().gc()
    }
}

impl Debug for $handle {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_tuple(stringify!($handle))
            .field(&*self.inner.$acquire())
            .finish()
    }
}
    };
}

define_cache!(LocalCache, Rc, RefCell::borrow_mut);
define_cache!(Cache: Send, Arc, Mutex::lock);

struct Namespace<Scope, Input, Output> {
    inner: HashMap<Scope, (Liveness, Input, Output)>,
}

impl<Scope, Input, Output> Namespace<Scope, Input, Output>
where
    Scope: Eq + Hash + 'static,
    Input: 'static,
    Output: 'static,
{
    fn get_if_input_eq<Query, Arg>(&mut self, query: &Query, input: &Arg) -> Option<&Output>
    where
        Query: Eq + Hash + ?Sized,
        Scope: Borrow<Query>,
        Arg: PartialEq<Input> + ?Sized,
        Input: Borrow<Arg>,
    {
        let (ref mut liveness, ref stored_input, ref stored) = self.inner.get_mut(query)?;
        if input == stored_input {
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

/// Describes the outcome for a cached value if a garbage collection were
/// to occur when observed. During the run of a `Revision` any cached values
/// which are initialized or read are marked as `Live`. At the end of a
/// `Revision`,
#[derive(Debug, PartialEq)]
enum Liveness {
    /// The value would be retained in a GC right now.
    Live,
    /// The value would be dropped in a GC right now.
    Dead,
}
