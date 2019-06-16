use crate::Point;
use std::any::{Any, TypeId};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct Env {
    inner: im::HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Env {
    pub(super) fn child(&self, additional: Env) -> Env {
        let mut new = self.clone();
        for (k, v) in additional.inner {
            new.inner.insert(k, v);
        }
        new
    }

    #[doc(hidden)]
    pub fn __set<E>(&mut self, e: E)
    where
        E: Any + Send + Sync + 'static,
    {
        self.inner.insert(TypeId::of::<E>(), Arc::new(e));
    }
}

pub fn get<E>() -> Option<impl Deref<Target = E> + 'static>
where
    E: Any + Send + Sync + 'static,
{
    Point::current()
        .env
        .inner
        .get(&TypeId::of::<E>())
        .map(|guard| {
            owning_ref::OwningRef::new(guard.to_owned()).map(|anon| anon.downcast_ref().unwrap())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn call_env() {
        let (mut first_called, mut second_called) = (false, false);
        let (first_byte, second_byte) = (0u8, 1u8);

        call!(|| {
            let curr_byte: u8 = *get::<u8>().unwrap();
            assert_eq!(curr_byte, first_byte);
            first_called = true;

            call!(|| {
                let curr_byte: u8 = *get::<u8>().unwrap();
                assert_eq!(curr_byte, second_byte);
                second_called = true;
            }, Env {
                u8 => second_byte
            });

            assert!(second_called);
            assert_eq!(curr_byte, first_byte);
        }, Env {
            u8 => first_byte
        });
        assert!(first_called);
        assert!(env::get::<u8>().is_none());
    }

}
