#![deny(clippy::all)]
#![feature(async_await, gen_future)]

#[macro_use]
pub extern crate tokio_trace;

use {
    futures::task::{LocalSpawn, SpawnError},
    std::{
        any::Any,
        future::Future,
        ops::{Deref, DerefMut},
        panic::{catch_unwind, AssertUnwindSafe, UnwindSafe},
        task::Waker,
    },
    topo::topo,
};

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

thread_local!(static WAKER: Waker = null_waker());
thread_local!(static SPAWNER: Box<dyn Spawn + Send> = null_spawner());

pub async fn runloop<C: Component + Clone + 'static>(
    root: C,
    spawner: impl Spawn + Send + 'static,
) {
    // make sure we can be woken back up and exited
    // std::future::get_task_context(|cx| WAKER.set(cx.waker().clone()));
    // SPAWNER.set(Box::new(spawner));

    loop {
        let root = AssertUnwindSafe(root.clone());
        if let Err(e) = catch_unwind(move || {
            let root = root.clone();
            trace!("composing");
            show!(root);
        }) {
            error!("error composing: {:?}", e);
            // TODO soft restart (reset state, recordings, etc)
        }
        futures::pending!();
    }
}

pub trait Spawn: 'static {
    fn spawn_local(
        &mut self,
        fut: Box<dyn Future<Output = ()> + 'static>,
    ) -> Result<(), SpawnError>;
    fn child(&self) -> Box<dyn Spawn>;
}

impl<Exec> Spawn for Exec
where
    Exec: 'static + Clone + LocalSpawn,
{
    fn spawn_local(
        &mut self,
        fut: Box<dyn Future<Output = ()> + 'static>,
    ) -> Result<(), SpawnError> {
        LocalSpawn::spawn_local_obj(self, fut.into())
    }

    fn child(&self) -> Box<dyn Spawn> {
        Box::new(self.clone())
    }
}

fn null_waker() -> Waker {
    unimplemented!()
}

fn null_spawner() -> Box<dyn Spawn + Send> {
    unimplemented!()
}
