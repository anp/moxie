//! Asynchronous loading primitives.

use {
    crate::embed::Spawner,
    std::{future::Future, task::Poll},
};

/// TODO
#[allow(clippy::drop_ref)]
#[topo::nested]
#[illicit::from_env(spawner: &Spawner)]
pub fn load_with<Arg, Fut, Stored, Ret>(
    _arg: Arg,
    _init: impl FnOnce(&Arg) -> Fut,
    _with: impl FnOnce(&Stored) -> Ret,
) -> Poll<Ret>
where
    Arg: PartialEq + 'static,
    Fut: Future<Output = Stored>,
    Stored: 'static,
    Ret: 'static,
{
    drop(spawner); // prevents ice in nll while this is empty
    unimplemented!()
}

/// TODO
#[topo::nested]
pub fn load_once_with<Fut, Stored, Ret>(
    init: impl FnOnce() -> Fut,
    with: impl FnOnce(&Stored) -> Ret,
) -> Poll<Ret>
where
    Fut: Future<Output = Stored>,
    Stored: 'static,
    Ret: 'static,
{
    load_with!((), |()| init(), with)
}

/// TODO
#[topo::nested]
pub fn load_once<Fut, Stored>(init: impl FnOnce() -> Fut) -> Poll<Stored>
where
    Fut: Future<Output = Stored>,
    Stored: Clone + 'static,
{
    load_once_with!(init, Clone::clone)
}

/// TODO
#[topo::nested]
pub fn load<Arg, Fut, Stored>(arg: Arg, init: impl FnOnce(&Arg) -> Fut) -> Poll<Stored>
where
    Arg: PartialEq + 'static,
    Fut: Future<Output = Stored>,
    Stored: Clone + 'static,
{
    load_with!(arg, init, Clone::clone)
}
