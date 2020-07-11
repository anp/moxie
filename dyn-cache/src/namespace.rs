use super::{Gc, Liveness};
use hashbrown::{
    hash_map::{DefaultHashBuilder, RawEntryMut},
    HashMap,
};
use std::{
    any::type_name,
    borrow::Borrow,
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
};

/// A query key that was hashed as part of an initial lookup and which can be
/// used to store fresh values back to the cache.
#[derive(Clone, Copy, Debug)]
pub struct Hashed<K, H = DefaultHashBuilder> {
    key: K,
    hash: u64,
    hasher: PhantomData<H>,
}

/// A namespace stores all cached values for a particular query type.
pub(super) struct Namespace<Scope, Input, Output, H = DefaultHashBuilder> {
    inner: HashMap<Scope, CacheCell<Input, Output>, H>,
}

impl<Scope, Input, Output> Default for Namespace<Scope, Input, Output> {
    fn default() -> Self {
        Self { inner: Default::default() }
    }
}

impl<Scope, Input, Output, H> Namespace<Scope, Input, Output, H>
where
    Scope: Eq + Hash + 'static,
    Input: 'static,
    Output: 'static,
    H: BuildHasher,
{
    fn hashed<'k, Key>(&self, key: &'k Key) -> Hashed<&'k Key, H>
    where
        Key: Hash + ?Sized,
    {
        let mut hasher = self.inner.hasher().build_hasher();
        key.hash(&mut hasher);
        Hashed { key, hash: hasher.finish(), hasher: PhantomData }
    }

    fn entry<'k, Key>(
        &mut self,
        hashed: &Hashed<&'k Key, H>,
    ) -> RawEntryMut<Scope, CacheCell<Input, Output>, H>
    where
        Key: Eq + ?Sized,
        Scope: Borrow<Key>,
    {
        self.inner.raw_entry_mut().from_hash(hashed.hash, |q| q.borrow().eq(hashed.key))
    }

    pub(super) fn get_if_input_eq<'k, Key, Arg>(
        &mut self,
        key: &'k Key,
        input: &Arg,
    ) -> Result<&Output, Hashed<&'k Key, H>>
    where
        Key: Eq + Hash + ?Sized,
        Scope: Borrow<Key>,
        Arg: PartialEq<Input> + ?Sized,
        Input: Borrow<Arg>,
    {
        let hashed = self.hashed(key);
        if let RawEntryMut::Occupied(occ) = self.entry(&hashed) {
            occ.into_mut().get_if_input_eq(input).ok_or(hashed)
        } else {
            Err(hashed)
        }
    }

    pub(super) fn store<Key>(&mut self, hashed: Hashed<&Key, H>, input: Input, output: Output)
    where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: Borrow<Key>,
    {
        match self.entry(&hashed) {
            RawEntryMut::Occupied(occ) => {
                occ.into_mut().store(input, output);
            }
            RawEntryMut::Vacant(vac) => {
                vac.insert(hashed.key.to_owned(), CacheCell::new(input, output));
            }
        }
    }
}

impl<Scope, Input, Output> Gc for Namespace<Scope, Input, Output>
where
    Scope: Eq + Hash + 'static,
    Input: 'static,
    Output: 'static,
{
    fn gc(&mut self) -> Liveness {
        self.inner.retain(|_, c| matches!(c.gc(), Liveness::Live));
        Liveness::Live // no reason to throw away the allocations behind namespaces afaict
    }
}

impl<Scope, Input, Output> Debug for Namespace<Scope, Input, Output>
where
    Scope: Eq + Hash + 'static,
    Input: 'static,
    Output: 'static,
{
    // someday specialization might save us from these lame debug impls?
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_map()
            .entry(&"scope", &type_name::<Scope>())
            .entry(&"input", &type_name::<Input>())
            .entry(&"output", &type_name::<Output>())
            .finish()
    }
}

/// A CacheCell represents the storage used for a particular input/output pair
/// on the heap.
struct CacheCell<Input, Output> {
    liveness: Liveness,
    input: Input,
    output: Output,
}

impl<Input, Output> CacheCell<Input, Output> {
    fn new(input: Input, output: Output) -> Self {
        Self { liveness: Liveness::Live, input, output }
    }

    /// Return a reference to the output if the input is equal, marking it live
    /// in the process.
    fn get_if_input_eq<Arg>(&mut self, input: &Arg) -> Option<&Output>
    where
        Arg: PartialEq<Input> + ?Sized,
        Input: Borrow<Arg>,
    {
        if input == &self.input {
            self.liveness = Liveness::Live;
            Some(&self.output)
        } else {
            None
        }
    }

    /// Store a new input/output and mark the storage live.
    fn store(&mut self, input: Input, output: Output) {
        self.liveness = Liveness::Live;
        self.input = input;
        self.output = output;
    }
}

impl<Input, Output> Gc for CacheCell<Input, Output>
where
    Input: 'static,
    Output: 'static,
{
    /// Always marks itself as dead in a GC, returning its previous value.
    fn gc(&mut self) -> Liveness {
        std::mem::replace(&mut self.liveness, Liveness::Dead)
    }
}

impl<Input, Output> Debug for CacheCell<Input, Output>
where
    Input: 'static,
    Output: 'static,
{
    // someday specialization might save us from these lame debug impls?
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_map()
            .entry(&"input", &type_name::<Input>())
            .entry(&"output", &type_name::<Output>())
            .finish()
    }
}
