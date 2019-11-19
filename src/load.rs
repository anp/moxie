//! Asynchronous loading primitives.

use {
    crate::{embed::Spawner, memo::*, state::*},
    futures::future::{AbortHandle, Abortable},
    std::{future::Future, task::Poll},
};

/// TODO
#[topo::nested]
#[illicit::from_env(spawner: &Spawner)]
pub fn load_with<Arg, Fut, Stored, Ret>(
    arg: Arg,
    init: impl FnOnce(&Arg) -> Fut,
    with: impl FnOnce(&Stored) -> Ret,
) -> Poll<Ret>
where
    Arg: PartialEq + 'static,
    Fut: Future<Output = Stored> + 'static,
    Stored: 'static,
    Ret: 'static,
{
    let result: Key<Poll<Stored>> = memo_state!((), |()| Poll::Pending);
    let result2 = result.clone();
    memo_with!(
        arg,
        |arg| {
            let (cancel, register_cancel) = AbortHandle::new_pair();
            let fut = init(arg);
            spawner
                .0
                .spawn_local_obj(
                    Box::pin(async move {
                        if let Ok(to_store) = Abortable::new(fut, register_cancel).await {
                            result.update(|_| Some(Poll::Ready(to_store)));
                        }
                    })
                    .into(),
                )
                .expect("spawner should definitely be live while in a revision");
            scopeguard::guard(cancel, |c| c.abort())
        },
        |_| {}
    );

    match &*result2 {
        Poll::Ready(ref stored) => Poll::Ready(with(stored)),
        Poll::Pending => Poll::Pending,
    }
}

/// TODO
#[topo::nested]
pub fn load_once_with<Fut, Stored, Ret>(
    init: impl FnOnce() -> Fut,
    with: impl FnOnce(&Stored) -> Ret,
) -> Poll<Ret>
where
    Fut: Future<Output = Stored> + 'static,
    Stored: 'static,
    Ret: 'static,
{
    load_with!((), |()| init(), with)
}

/// TODO
#[topo::nested]
pub fn load_once<Fut, Stored>(init: impl FnOnce() -> Fut) -> Poll<Stored>
where
    Fut: Future<Output = Stored> + 'static,
    Stored: Clone + 'static,
{
    load_with!((), |()| init(), Clone::clone)
}

/// TODO
#[topo::nested]
pub fn load<Arg, Fut, Stored>(arg: Arg, init: impl FnOnce(&Arg) -> Fut) -> Poll<Stored>
where
    Arg: PartialEq + 'static,
    Fut: Future<Output = Stored> + 'static,
    Stored: Clone + 'static,
{
    load_with!(arg, init, Clone::clone)
}
