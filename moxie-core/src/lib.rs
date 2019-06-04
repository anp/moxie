#![deny(clippy::all)]
#![feature(await_macro, async_await, const_fn, gen_future, weak_ptr_eq)]

#[macro_use]
extern crate execution_context;
#[macro_use]
pub extern crate tokio_trace;

use {
    std::{
        any::Any,
        future::Future,
        ops::{Deref, DerefMut},
        panic::UnwindSafe,
    },
    topo::topo,
};

pub use crate::task::{runloop, Spawn};

#[doc(hidden)]
pub use tokio_trace as __trace;

pub trait Component {
    fn content(self);
}

#[topo]
fn memo<T>(initializer: impl FnOnce() -> T) -> T
where
    T: Clone + 'static,
{
    // TODO memoize it!
    initializer()
}

#[topo]
pub fn show(_child: impl Component) {
    unimplemented!()
}

#[topo]
pub fn state<S: 'static + Any + UnwindSafe>(_init: impl FnOnce() -> S) -> Guard<S> {
    unimplemented!()
}

#[topo]
pub fn task(_fut: impl Future<Output = ()> + Send + UnwindSafe + 'static) {
    unimplemented!()
}

#[topo]
pub fn record<N>(_node: N)
where
    N: 'static,
{
    unimplemented!()
}

pub struct Guard<State> {
    // TODO
    _ty: std::marker::PhantomData<State>,
}

impl<State> Guard<State> {
    pub fn key(&self) -> Key<State> {
        unimplemented!()
    }
}

impl<State> Deref for Guard<State> {
    type Target = State;
    fn deref(&self) -> &Self::Target {
        unimplemented!()
    }
}

impl<State> DerefMut for Guard<State> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unimplemented!()
    }
}

pub struct Key<State> {
    _ty: std::marker::PhantomData<State>,
}

// at bottom bc macros?
mod task;
