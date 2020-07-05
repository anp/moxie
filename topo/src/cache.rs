//! Caches for storing the results of repeated queries.
//!
//! Every query to a cache has a "scope" by which it is namespaced. The types in
//! this module cache queries which have arbitrary scope types, storing one
//! input value and one ouput value per scope:
//!
//! ```
//! let storage = topo::cache::SharedLocalCache::default();
//! let count = std::cell::Cell::new(0);
//! let increment = |n| {
//!     storage.cache_with(
//!         "increment by arg",
//!         &n,
//!         |&n| {
//!             let new_count = count.get() + n;
//!             count.set(new_count);
//!             new_count
//!         },
//!         Clone::clone,
//!     )
//! };
//!
//! assert_eq!(count.get(), 0);
//!
//! assert_eq!(increment(2), 2);
//! assert_eq!(count.get(), 2);
//!
//! // running the query again with the same input has no external effect
//! assert_eq!(increment(2), 2);
//! assert_eq!(count.get(), 2);
//!
//! // same query with a different input will run
//! assert_eq!(increment(1), 3);
//! assert_eq!(count.get(), 3);
//!
//! // these all have the same scope, so this runs again
//! assert_eq!(increment(3), 6);
//! assert_eq!(count.get(), 6);
//!
//! let decrement = |n| {
//!     storage.cache_with(
//!         "decrement by arg",
//!         &n,
//!         |&n| {
//!             let new_count = count.get() - n;
//!             count.set(new_count);
//!             new_count
//!         },
//!         Clone::clone,
//!     )
//! };
//!
//! assert_eq!(decrement(1), 5);
//! assert_eq!(count.get(), 5);
//!
//! // increment's last query is still cached
//! assert_eq!(increment(3), 6);
//! assert_eq!(count.get(), 5);
//!
//! // this one is still cached too, because they're different queries
//! assert_eq!(decrement(1), 5);
//! assert_eq!(count.get(), 5);
//! ```

pub(crate) mod token;

use downcast_rs::{impl_downcast, Downcast};
use hash_hasher::HashedMap;
use parking_lot::Mutex;
use std::{
    any::{type_name, TypeId},
    borrow::Borrow,
    cell::RefCell,
    cmp::Eq,
    collections::HashMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::Hash,
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

macro_rules! doc_comment {
    ($($contents:expr)+ => $($item:tt)+) => {
        doc_comment! {@ concat!($($contents),+), $($item)+ }
    };
    (@ $contents:expr, $($item:tt)+) => {
        #[doc = $contents]
        $($item)+
    };
}

macro_rules! define_cache {
    ($name:ident $(: $bound:ident)?, $($rest:tt)*) => {
doc_comment! {"
Holds arbitrary query results which are namespaced by arbitrary scope types. Usually used
through [`Shared" stringify!($name) "::cache_with`] and [`Shared" stringify!($name) "::gc`].

# Query types

> Note: the types referenced in this documentation are only visible on individual methods, as
> `" stringify!($name) "` is not itself a generic type.

Storage is sharded by the type of the query. The type of a query has three parts:
 
The query scope is the value which indexes the storage for a particular query type, it has the
bound `Scope: 'static + Eq + Hash" $(" + " stringify!($bound))? "`.

Each `Scope` corresponds to at most a single `Input: 'static" $(" + " stringify!($bound))? "`
and a single `Output: 'static" $(" + " stringify!($bound))? "` value at any given time.

# Reading stored values

See [`" stringify!($name) "::get_if_arg_eq_prev_input`] which accepts borrowed forms of `Scope`
and `Input`: `Key` and `Arg` respectively. `Arg` must satisfy `PartialEq<Input>` to determine
whether to return a stored output.

# Garbage Collection

Each time [`" stringify!($name) "::gc`] is called it removes any values which haven't been
referenced since the prior call.

After each GC, all values still in the cache are marked garbage. They are marked live again when
inserted with [`" stringify!($name) "::store`] or read with
[`" stringify!($name) "::get_if_arg_eq_prev_input`].
"=>
#[derive(Debug, Default)]
pub struct $name {
    /// We use a [`hash_hasher::HashedMap`] here because we know that `Query` is made up only of
    /// `TypeIds` which come pre-hashed courtesy of rustc.
    inner: HashedMap<QueryTy, Box<dyn Gc $(+ $bound)?>>,
}}

impl $name {
    /// Return a reference to a query's stored output if a result is stored and `arg` equals the
    /// previously-stored `Input`. If a reference is returned, the stored input/output
    /// is marked live and will not be GC'd the next call.
    pub fn get_if_arg_eq_prev_input<Key, Scope, Arg, Input, Output>(
        &mut self,
        key: &Key,
        arg: &Arg,
    ) -> Option<&Output>
    where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: 'static + Borrow<Key> + Eq + Hash $(+ $bound)?,
        Arg: PartialEq<Input> + ?Sized,
        Input: 'static + Borrow<Arg> $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        self.get_namespace_mut::<Scope, Input, Output>().get_if_input_eq(key, arg)
    }

    /// Stores the input/output of a query which will not be GC'd at the next call.
    pub fn store<Key, Scope, Input, Output>(
        &mut self,
        key: &Key,
        input: Input,
        output: Output,
    ) where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: 'static + Borrow<Key> + Eq + Hash $(+ $bound)?,
        Input: 'static $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        self.get_namespace_mut().store(key, input, output);
    }

    fn get_namespace_mut<Scope, Input, Output>(&mut self) -> &mut Namespace<Scope, Input, Output>
    where
        Scope: 'static + Eq + Hash $(+ $bound)?,
        Input: 'static $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        let key: Query<Scope, Input, Output> = Query::new();
        let gc: &mut (dyn Gc $(+ $bound)?) = &mut **self
            .inner
            .entry(key.ty())
            .or_insert_with(|| Box::new(Namespace::<Scope, Input, Output>::default()));
        gc.as_any_mut().downcast_mut().unwrap()
    }

    /// Drops any values which were not referenced since the last call to this method.
    pub fn gc(&mut self) {
        for namespace in self.inner.values_mut() {
            namespace.gc();
        }
    }
}

impl std::panic::UnwindSafe for $name {}
impl std::panic::RefUnwindSafe for $name {}

paste::item! {
    define_cache! {
        @handle $name $(: $bound)?, [<Shared $name>], $($rest)*
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

doc_comment! {"
Provides shared, synchronized access to a [`" stringify!($name) "`] and a function-memoization
API in [`" stringify!($handle) "::cache_with`].

# Example

```
let storage = topo::cache::" stringify!($handle) r#"::default();
let call_count = std::cell::Cell::new(0);
let increment_count = |&to_add: &i32| {
    let new_count = call_count.get() + to_add;
    call_count.set(new_count);
    new_count
};

assert_eq!(call_count.get(), 0, "not called yet");

let with_one = storage.cache_with(&'a', &1, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 1, "called only once");
assert_eq!(call_count.get(), with_one);

let with_one_again = storage.cache_with(&'a', &1, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 1, "still called only once, previous value cached");
assert_eq!(call_count.get(), with_one_again);

let with_two = storage.cache_with(&'a', &2, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 3, "called again with a new, larger increment");
assert_eq!(call_count.get(), with_two);

let with_other_query = storage.cache_with(&'b', &1, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 4, "called again with the same increment, different scope");
assert_eq!(call_count.get(), with_other_query);

let with_two_again = storage.cache_with(&'a', &2, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 4, "cell still has last mutation's value");
assert_eq!(with_two_again, with_two, "cache should still have previous value");

storage.gc(); // won't drop any values, but sets all of the cached values to be dropped
call_count.set(0);

// re-run 'b', marking it live
let reran_other_query = storage.cache_with(&'b', &1, &increment_count, Clone::clone);
assert_eq!(reran_other_query , 4, "returns the cached value");
assert_eq!(call_count.get(), 0, "without running increment_count");

storage.gc(); // query 'a' will be dropped

// re-run 'b', observing cached value
let reran_other_query = storage.cache_with(&'b', &1, &increment_count, Clone::clone);
assert_eq!(reran_other_query , 4, "still returns the cached value");
assert_eq!(call_count.get(), 0, "still without running increment_count");

// run 'a' again, observe no cached value
let with_one_again = storage.cache_with(&'a', &1, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 1, "called without caching");
assert_eq!(call_count.get(), with_one_again);
```
"#=>
#[derive(Clone)]
pub struct $handle {
    inner: $shared<$lock<$name>>,
}}

impl Default for $handle {
    fn default() -> Self {
        Self {
            inner: $shared::new($lock::new($name::default()))
        }
    }
}

impl $handle {
    /// Caches the result of `init(arg)` once per `key`, re-running it when `arg` changes. Always
    /// runs `with` on the stored `Output` before returning the result.
    ///
    /// See the [moxie](https://docs.rs/moxie) crate for ergonomic wrappers.
    pub fn cache_with<Key, Scope, Arg, Input, Output, Ret>(
        &self,
        key: &Key,
        arg: &Arg,
        init: impl FnOnce(&Input) -> Output,
        with: impl FnOnce(&Output) -> Ret,
    ) -> Ret
    where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: 'static + Borrow<Key> + Eq + Hash $(+ $bound)?,
        Arg: PartialEq<Input> + ToOwned<Owned=Input> + ?Sized,
        Input: 'static + Borrow<Arg> $(+ $bound)?,
        Output: 'static $(+ $bound)?,
        Ret: 'static $(+ $bound)?,
    {
        if let Some(stored) = { self.inner.$acquire().get_if_arg_eq_prev_input(key, arg) } {
            return with(stored);
        }

        let arg = arg.to_owned();
        let to_store = init(&arg);
        let to_return = with(&to_store);
        self.inner.$acquire().store(key, arg, to_store);
        to_return
    }

doc_comment!{"
Forwards to [`" stringify!($name) "::gc`].
"=>
    pub fn gc(&self) {
        self.inner.$acquire().gc()
    }}
}

impl Debug for $handle {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_tuple(stringify!($handle))
            .field(&*self.inner.$acquire())
            .finish()
    }
}

impl From<$name> for $handle {
    fn from(inner: $name) -> Self {
        Self { inner: $shared::new($lock::new(inner)) }
    }
}

impl std::panic::UnwindSafe for $handle {}
impl std::panic::RefUnwindSafe for $handle {}
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
    fn get_if_input_eq<Key, Arg>(&mut self, key: &Key, input: &Arg) -> Option<&Output>
    where
        Key: Eq + Hash + ?Sized,
        Scope: Borrow<Key>,
        Arg: PartialEq<Input> + ?Sized,
        Input: Borrow<Arg>,
    {
        let (ref mut liveness, ref stored_input, ref stored) = self.inner.get_mut(key)?;
        if input == stored_input {
            *liveness = Liveness::Live;
            Some(stored)
        } else {
            None
        }
    }

    fn store<Key>(&mut self, key: &Key, input: Input, output: Output)
    where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: Borrow<Key>,
    {
        if let Some((liveness, prev_input, prev_output)) = self.inner.get_mut(key) {
            *liveness = Liveness::Live;
            *prev_input = input;
            *prev_output = output;
        } else {
            let scope = key.to_owned();
            self.inner.insert(scope, (Liveness::Live, input, output));
        }
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

/// The "composite key" for a [`Query`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
struct QueryTy(TypeId);

/// Each query has an `Input`, and an `Output` which together can be
/// thought of as defining a function: `(input) -> output`.
struct Query<Scope, Input, Output> {
    /// The type of scope by which the query is namespaced.
    scope: PhantomData<Scope>,
    /// The type of input the query accepts.
    input: PhantomData<Input>,
    /// The type of output the query returns.
    output: PhantomData<Output>,
}

impl<Scope, Input, Output> Query<Scope, Input, Output>
where
    Scope: 'static,
    Input: 'static,
    Output: 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    fn ty(&self) -> QueryTy {
        QueryTy(TypeId::of::<Self>())
    }
}

impl<Scope, Input, Output> Default for Query<Scope, Input, Output>
where
    Scope: 'static,
    Input: 'static,
    Output: 'static,
{
    fn default() -> Self {
        Self { scope: PhantomData, input: PhantomData, output: PhantomData }
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
