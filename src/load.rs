//! Asynchronous loading primitives.

use {
    crate::{embed::Spawner, memo::memo_with, state::*},
    futures::future::{AbortHandle, Abortable},
    std::{future::Future, task::Poll},
};

/// Load a value from the future returned by `init` whenever `capture` changes, returning the
/// result of calling `with` with the loaded value. Cancels the running future after any revision
/// during which this call was not made.
#[topo::nested]
#[illicit::from_env(spawner: &Spawner)]
pub fn load_with<Arg, Fut, Stored, Ret>(
    capture: Arg,
    init: impl FnOnce(&Arg) -> Fut,
    with: impl FnOnce(&Stored) -> Ret,
) -> Poll<Ret>
where
    Arg: PartialEq + 'static,
    Fut: Future<Output = Stored> + 'static,
    Stored: 'static,
    Ret: 'static,
{
    let result: Key<Poll<Stored>> = memo_state((), |()| Poll::Pending);
    let result2 = result.clone();
    memo_with(
        capture,
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
        |_| {},
    );

    match &*result2 {
        Poll::Ready(ref stored) => Poll::Ready(with(stored)),
        Poll::Pending => Poll::Pending,
    }
}

/// Calls [`load_with`] but never re-initializes the loading future.
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
    load_with((), |()| init(), with)
}

/// Calls [`load_with`], never re-initializes the loading future, and clones the returned value
/// on each revision once the future has completed and returned.
#[topo::nested]
pub fn load_once<Fut, Stored>(init: impl FnOnce() -> Fut) -> Poll<Stored>
where
    Fut: Future<Output = Stored> + 'static,
    Stored: Clone + 'static,
{
    load_with((), |()| init(), Clone::clone)
}

/// Load a value from a future, cloning it on subsequent revisions after it is first returned.
/// Re-initializes the loading future if the capture argument changes from previous revisions.
#[topo::nested]
pub fn load<Arg, Fut, Stored>(capture: Arg, init: impl FnOnce(&Arg) -> Fut) -> Poll<Stored>
where
    Arg: PartialEq + 'static,
    Fut: Future<Output = Stored> + 'static,
    Stored: Clone + 'static,
{
    load_with(capture, init, Clone::clone)
}

#[cfg(test)]
mod tests {
    use {super::*, std::rc::Rc};

    #[test]
    fn basic_loading_phases() {
        let (send, recv) = futures::channel::oneshot::channel();
        // this is uh weird, but we know up front how much we'll poll this
        let recv = Rc::new(futures::lock::Mutex::new(Some(recv)));

        let mut rt = crate::embed::Runtime::new(move || -> Poll<u8> {
            let recv = recv.clone();
            load_once(|| {
                async move {
                    recv.lock()
                        .await
                        .take()
                        .expect("load_once should only allow us to take from the option once")
                        .await
                        .expect("we control the channel and won't drop it")
                }
            })
        });

        assert_eq!(
            rt.run_once(),
            Poll::Pending,
            "no values received when nothing sent"
        );
        assert_eq!(
            rt.run_once(),
            Poll::Pending,
            "no values received, and we aren't blocking"
        );

        send.send(5u8).unwrap();
        assert_eq!(
            rt.run_once(),
            Poll::Ready(5),
            "we need to receive the value we sent"
        );
        assert_eq!(
            rt.run_once(),
            Poll::Ready(5),
            "the value we sent must be cached because its from a oneshot channel"
        );
    }

    #[test]
    fn interest_loss_cancels_task() {
        let (send, recv) = futures::channel::oneshot::channel();
        let recv = Rc::new(futures::lock::Mutex::new(Some(recv)));

        let mut rt = crate::embed::Runtime::new(move || -> Option<Poll<u8>> {
            if crate::embed::Revision::current().0 < 3 {
                let recv = recv.clone();
                Some(load_once(|| {
                    async move {
                        recv.lock()
                            .await
                            .take()
                            .expect("load_once should only allow us to take from the option once")
                            .await
                            .expect("we control the channel and won't drop it")
                    }
                }))
            } else {
                None
            }
        });

        assert_eq!(rt.run_once(), Some(Poll::Pending));
        assert!(
            !send.is_canceled(),
            "interest expressed, receiver must be live"
        );

        assert_eq!(rt.run_once(), Some(Poll::Pending));
        assert!(
            !send.is_canceled(),
            "interest still expressed, receiver must be live"
        );

        assert_eq!(rt.run_once(), None);
        assert!(
            !send.is_canceled(),
            "interest dropped, task live for another revision"
        );

        assert_eq!(rt.run_once(), None);
        assert!(send.is_canceled(), "interest dropped, task dropped");

        assert!(
            send.send(4u8).is_err(),
            "must be no task holding the channel and able to receive a message"
        );
    }
}
