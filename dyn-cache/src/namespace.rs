use super::{
    cache_cell::CacheCell,
    dep_node::{DepNode, Dependent},
    Storage,
};
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

/// The result of failing to find a `key` in a cache with matching input. Passed
/// back to [`Namespace::store`] to initialize a value in the cache.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KeyMiss<'k, K: ?Sized, I, H> {
    inner: Result<Hashed<&'k K, H>, &'k K>,
    dependent: Dependent,
    node: Option<DepNode>,
    input: I,
}

impl<'k, K: ?Sized, I, H> KeyMiss<'k, K, I, H> {
    fn hashed(h: Hashed<&'k K, H>, input: I, node: Option<DepNode>, dependent: Dependent) -> Self {
        Self { inner: Ok(h), node, dependent, input }
    }

    pub(crate) fn just_key(k: &'k K, input: I, dependent: Dependent) -> Self {
        let node = DepNode::new(dependent);
        let dependent = node.as_dependent();
        Self { inner: Err(k), dependent, node: Some(node), input }
    }

    pub(crate) fn init<R>(&self, op: impl FnOnce(&I) -> R) -> R {
        self.dependent.clone().init_dependency(|| op(&self.input))
    }
}

impl<'k, K, I, H> Debug for KeyMiss<'k, K, I, H>
where
    K: Debug + ?Sized,
    I: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("KeyMiss")
            .field("inner", &self.inner)
            .field("dependent", &self.dependent)
            .field("node", &self.node)
            .field("input", &self.input)
            .finish()
    }
}

/// A query key that was hashed as part of an initial lookup and which can be
/// used to store fresh values back to the cache.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Hashed<K, H> {
    key: K,
    hash: u64,
    hasher: PhantomData<H>,
}

impl<K, H> Debug for Hashed<K, H>
where
    K: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Hashed")
            .field("key", &self.key)
            .field("hash", &self.hash)
            .field("hasher", &std::any::type_name::<H>())
            .finish()
    }
}

/// A namespace stores all cached values for a particular query type.
#[derive(Clone)]
pub(crate) struct Namespace<Scope, Input, Output, H = DefaultHashBuilder> {
    inner: HashMap<Scope, CacheCell<Input, Output>, H>,
}

impl<Scope, Input, Output, H> Default for Namespace<Scope, Input, Output, H>
where
    H: Default,
{
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
        &self,
        hashed: &Hashed<&'k Key, H>,
    ) -> Option<(&Scope, &CacheCell<Input, Output>)>
    where
        Key: Eq + ?Sized,
        Scope: Borrow<Key>,
    {
        self.inner.raw_entry().from_hash(hashed.hash, |q| q.borrow().eq(hashed.key))
    }

    fn entry_mut<'k, Key>(
        &mut self,
        hashed: &Hashed<&'k Key, H>,
    ) -> RawEntryMut<Scope, CacheCell<Input, Output>, H>
    where
        Key: Eq + ?Sized,
        Scope: Borrow<Key>,
    {
        self.inner.raw_entry_mut().from_hash(hashed.hash, |q| q.borrow().eq(hashed.key))
    }

    pub fn get<'k, Key, Arg>(
        &self,
        key: &'k Key,
        arg: &Arg,
        dependent: Dependent,
    ) -> Result<&Output, KeyMiss<'k, Key, Input, H>>
    where
        Key: Eq + Hash + ?Sized,
        Scope: Borrow<Key>,
        Arg: PartialEq<Input> + ToOwned<Owned = Input> + ?Sized,
        Input: Borrow<Arg>,
    {
        let hashed = self.hashed(key);
        if let Some((_, cell)) = self.entry(&hashed) {
            cell.get(arg, dependent).map_err(|d| KeyMiss::hashed(hashed, arg.to_owned(), None, d))
        } else {
            let node = DepNode::new(dependent);
            let new_dep = node.as_dependent();
            Err(KeyMiss::hashed(hashed, arg.to_owned(), Some(node), new_dep))
        }
    }

    pub fn store<Key>(&mut self, miss: KeyMiss<'_, Key, Input, H>, output: Output)
    where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: Borrow<Key>,
    {
        let dependent = miss.dependent;
        let hashed = miss.inner.unwrap_or_else(|k| self.hashed(k));
        match self.entry_mut(&hashed) {
            RawEntryMut::Occupied(occ) => {
                assert!(miss.node.is_none(), "mustn't create nodes that aren't used");
                occ.into_mut().store(miss.input, output, dependent);
            }
            RawEntryMut::Vacant(vac) => {
                vac.insert(
                    hashed.key.to_owned(),
                    CacheCell::new(
                        miss.input,
                        output,
                        miss.node.expect("if no cell present, we must have created a fresh node"),
                    ),
                );
            }
        }
    }
}

impl<Scope, Input, Output, H> Storage for Namespace<Scope, Input, Output, H>
where
    Scope: 'static,
    Input: 'static,
    Output: 'static,
    H: 'static,
{
    fn mark(&mut self) {
        self.inner.values_mut().for_each(CacheCell::update_liveness);
    }

    fn sweep(&mut self) {
        self.inner.retain(|_, c| {
            let keep = c.is_live();
            c.mark_dead();
            keep
        });
    }
}

impl<Scope, Input, Output, H> Debug for Namespace<Scope, Input, Output, H> {
    // someday specialization might save us from these lame debug impls?
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        // TODO(#176) better debug output somehow?
        f.debug_map()
            .entry(&"scope", &type_name::<Scope>())
            .entry(&"input", &type_name::<Input>())
            .entry(&"output", &type_name::<Output>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_debug_output() {
        let ns: Namespace<u8, u8, u8> = Default::default();
        let output = format!("{:?}", ns);
        assert_eq!(output, "{\"scope\": \"u8\", \"input\": \"u8\", \"output\": \"u8\"}");
    }
}
