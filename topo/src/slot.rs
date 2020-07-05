use crate::cache::Cache;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{
    any::{type_name, TypeId},
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    marker::PhantomData,
};

static TOKENS: Lazy<Mutex<Cache>> = Lazy::new(|| Mutex::new(Cache::default()));

/// A unique identifer in the global cache. Each type can have
/// [`std::u32::MAX`] unique values cached. Constructed with [`Token::make`],
/// which will always produce the same value for the same input.
///
/// # Memory Usage
///
/// Token inputs are not yet dropped and care should be taken when creating
/// large numbers of them, as the memory used over time will grow with
/// proportion to the number of unique tokens created.
///
/// See [issue #141](https://github.com/anp/moxie/issues/141) for future work.
///
/// A typed token can be converted into an [`OpaqueToken`] to allow
/// differentiating between unique values of different types.
pub(crate) struct Token<T> {
    index: u32,
    ty: PhantomData<T>,
}

impl<T> Token<T>
where
    T: Eq + Hash + Send + 'static,
{
    /// Makes a unique token from the provided value, interning it in the global
    /// cache. Later calls with the same input will return the same token.
    pub fn make<Q>(value: &Q) -> Token<T>
    where
        Q: Eq + Hash + ToOwned<Owned = T> + ?Sized,
        T: Borrow<Q>,
    {
        static INDICES: Lazy<Mutex<HashMap<TypeId, u32>>> =
            Lazy::new(|| Mutex::new(HashMap::new()));
        let mut existing_tokens = TOKENS.lock();

        if let Some(token) = existing_tokens.get_if_arg_eq_prev_input(value, &()) {
            *token
        } else {
            let mut indices = INDICES.lock();
            let count = indices.entry(TypeId::of::<T>()).or_default();
            *count += 1;
            let new_token = Self { index: *count, ty: PhantomData };
            existing_tokens.store(value, (), new_token);
            new_token
        }
    }

    /// Fabricate a token. Used for e.g. creating a root `crate::CallId`.
    pub(crate) fn fake() -> Self {
        Self { index: 0, ty: PhantomData }
    }
}

impl<T> Clone for Token<T> {
    fn clone(&self) -> Self {
        Self { index: self.index, ty: PhantomData }
    }
}

impl<T> Copy for Token<T> {}

impl<T> Debug for Token<T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Token").field("index", &self.index).field("ty", &type_name::<T>()).finish()
    }
}

impl<Q, T> From<&Q> for Token<T>
where
    Q: Eq + Hash + ToOwned<Owned = T> + ?Sized,
    T: Borrow<Q> + Eq + Hash + Send + 'static,
{
    fn from(query: &Q) -> Self {
        Token::make(query)
    }
}

impl<T> Hash for Token<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.index.hash(hasher)
    }
}

impl<T> PartialEq for Token<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for Token<T> {}

impl<T> PartialOrd for Token<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}
impl<T> Ord for Token<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

/// A unique type-erased identifier for a cached value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub(crate) struct OpaqueToken {
    ty: TypeId,
    index: u32,
}

impl<T: 'static> From<Token<T>> for OpaqueToken {
    fn from(token: Token<T>) -> Self {
        OpaqueToken { index: token.index, ty: TypeId::of::<T>() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_tokens() {
        let foo: Token<String> = Token::make("foo");
        assert_eq!(foo, Token::make("foo"));
        assert_ne!(foo, Token::make("bar"));
    }

    #[test]
    fn make_opaque() {
        let first: OpaqueToken = Token::make(&10u8).into();
        let second: OpaqueToken = Token::make(&10u16).into();
        assert_ne!(first, second);
    }
}
