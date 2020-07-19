#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs)]

//! Caches for storing the results of repeated function calls. The cache types
//! available use minimal dynamic dispatch to allow storing arbitrarily many
//! types of query results in a single parent store.
//!
//! There are two main flavors of cache available for use in this crate:
//!
//! | Mutable type          | Requires `Send`? |
//! |-----------------------|------------------|
//! | [`sync::SendCache`]   | yes              |
//! | [`local::LocalCache`] | no               |
//!
//! These "inner" caches require mutable access to call their functions like
//! [`local::LocalCache::get`] or [`sync::SendCache::store`].
//!
//! They each come available in a shared variant:
//!
//! | Shared type                 | Synchronized? |
//! |-----------------------------|---------------|
//! | [`sync::SharedSendCache`]   | Mutex         |
//! | [`local::SharedLocalCache`] | RefCell       |
//!
//! These variants are wrapped with reference counting and synchronization and
//! are used by calling [`sync::SharedSendCache::cache_with`] or
//! [`local::SharedLocalCache::cache`].
//!
//! # Query types
//!
//! Each [`Query`] type maps to a typed "namespace" within the unityped cache
//! storage, each query having a distinct type for its scope, input, and
//! output.
//!
//! ## Scopes
//!
//! The scope of a query is its identifier within cache storage.
//! Scopes must implement `Eq` and `Hash` so that results can be
//! efficiently and uniquely indexed within a namespace.
//!
//! Each scope identifies 0-1 `(Input, Output)` pairs in each namespace. The
//! same type of scope can be used in multiple [`Query`]s without collision if
//! the types of inputs, outputs, or both differ.
//!
//! ## Inputs
//!
//! The input to a query determines when it is (re-)run. If a given query has
//! been run before, then the previous input is compared to the current input
//! before potentially running the query. If the input hasn't changed, the query
//! can be skipped and its previously-stored output is returned.
//!
//! ## Outputs
//!
//! The only constraint on query outputs is that they are owned (`Output:
//! 'static`). This imposes the inconvenient requirement that all access to
//! stored values occurs during the scope of a closure (similar to thread-locals
//! in the standard library).
//!
//! The most common way to work around this requirement is to choose output
//! types that cheaply implement [`std::clone::Clone`].
//!
//! # Allocations
//!
//! In order to store distinct query results in the same container, allocations
//! and indirection are required.
//!
//! ## Borrowed query parameters
//!
//! All of the cache functions accept a reference to a type `Key:
//! ToOwned<Owned=Scope>` so that the scope is only cloned on the first
//! insertion to its storage and all subsequent lookups can be with a borrowed
//! type.
//!
//! Like the query scope, functions to get cache values accept a borrowed
//! version of the input and only clone it when the input has changed.
//!
//! ## Causes
//!
//! There are three situations where these caches allocate:
//!
//! 1. caching new types which haven't been seen by that cache instance yet
//! 2. storing the results of a new query
//! 3. updating the results of a stored query
//!
//! There are several types of allocations performed by the caches in this
//! crate:
//!
//! | Allocation                         | Causes   |
//! |------------------------------------|----------|
//! | box a new, empty namespace         | (1)      |
//! | resize a cache's map of namespaces | (1)      |
//! | call `.to_owned()` on a scope/key  | (2)      |
//! | resize a namespace's storage       | (2)      |
//! | call `.to_owned()` on an input/arg | (2), (3) |
//!
//! Outside of these, only user-defined functions should perform any allocation.
//!
//! # Garbage Collection
//!
//! Every value in the cache has a [`Liveness`] which is set to
//! [`Liveness::Live`] when the value is first stored and again when it is read.
//!
//! The inner caches implement [`Gc`] and it is available through
//! [`local::SharedLocalCache::gc`] & [`sync::SharedSendCache::gc`]. When
//! called, the `gc()` method retains only those values which are still
//! [`Liveness::Live`] and then marks them all [`Liveness::Dead`] again.
//!
//! This behavior resembles a simple mark-and-sweep garbage collector where the
//! "mark phase" is the use of the cache in between [`Gc::gc`] calls. Any values
//! which weren't used in the mark phase are dropped in the next "sweep phase"
//! when [`Gc::gc`] is called.
//!
//! ## Nested Queries
//!
//! While it is possible to nest use of the shared caches within the init
//! closures passed to them, the caches do not yet track the required dependency
//! relationship to correctly retain intermediate cached results across GCs.
//! While this works well enough for some scenarios it needs to be resolved in
//! the general case before this way of using this crate is recommended.

use downcast_rs::{impl_downcast, Downcast};
use hash_hasher::HashBuildHasher;
use hashbrown::hash_map::DefaultHashBuilder;
use std::{
    any::TypeId,
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
};

#[macro_use]
mod definition;
mod storage;

pub use storage::Hashed;
use storage::Namespace;

/// A type which can contain values of varying liveness, including itself.
pub trait Gc: Downcast + Debug {
    /// Remove dead entries, returning the container's own status afterwards.
    fn gc(&mut self) -> Liveness;
}

impl_downcast!(Gc);

/// Describes the outcome of garbage collection for a cached value.
#[derive(Debug, PartialEq)]
pub enum Liveness {
    /// The value would be retained in a GC right now.
    Live,
    /// The value would be dropped in a GC right now.
    Dead,
}

/// The result of lookup up a query key and its input/arg within a cache.
type CacheLookup<'k, Key, Scope, Input, Output, H = DefaultHashBuilder> =
    (Query<Scope, Input, Output>, KeyLookup<'k, Key, H>);

/// The result of looking up a key within a cache namespace.
type KeyLookup<'k, K, H = DefaultHashBuilder> = Result<Hashed<&'k K, H>, &'k K>;

/// The type of a dynamic cache query, used to shard storage in a fashion
/// similar to `anymap` or `typemap`.
pub struct Query<Scope, Input, Output, H = HashBuildHasher> {
    ty: PhantomData<(Scope, Input, Output)>,
    hasher: PhantomData<H>,
    hash: u64,
}

impl<Scope, Input, Output, H> Query<Scope, Input, Output, H>
where
    Scope: 'static,
    Input: 'static,
    Output: 'static,
    H: BuildHasher,
{
    fn new(build: &H) -> Self {
        // this is a bit unrustic but it lets us keep the typeid defined once
        let mut new = Query { ty: PhantomData, hasher: PhantomData, hash: 0 };
        let mut hasher = build.build_hasher();
        new.ty().hash(&mut hasher);
        new.hash = hasher.finish();
        new
    }

    fn make_namespace(&self) -> Box<Namespace<Scope, Input, Output>> {
        Box::new(Namespace::default())
    }

    fn hash(&self) -> u64 {
        self.hash
    }

    fn ty(&self) -> TypeId {
        TypeId::of::<(Scope, Input, Output)>()
    }
}

/// A cache for types which are not thread-safe (`?Send`).
pub mod local {
    use super::*;
    use hash_hasher::HashBuildHasher;
    use hashbrown::HashMap;
    use std::{
        any::TypeId,
        borrow::Borrow,
        cell::RefCell,
        cmp::Eq,
        fmt::{Debug, Formatter, Result as FmtResult},
        hash::Hash,
        rc::Rc,
    };

    define_cache!(local, LocalCache, Rc, RefCell::borrow_mut);
}

/// A thread-safe cache which requires stored types implement `Send`.
pub mod sync {
    use super::*;
    use hash_hasher::HashBuildHasher;
    use hashbrown::HashMap;
    use parking_lot::Mutex;
    use std::{
        any::TypeId,
        borrow::Borrow,
        cmp::Eq,
        fmt::{Debug, Formatter, Result as FmtResult},
        hash::Hash,
        sync::Arc,
    };

    define_cache!(sync, SendCache: Send, Arc, Mutex::lock);
}
