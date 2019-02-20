use {
    crate::prelude::*,
    futures::future::FutureObj,
    std::{
        future::Future,
        task::{LocalWaker, Poll},
    },
};

/// Returns a `Future` which wakes an additional task each time the wrapped future's task is woken.
pub(crate) fn also_wake<F, R>(also: Waker, fut: F) -> AlsoWake<F, R>
where
    F: Future<Output = R> + Send,
{
    AlsoWake {
        fut,
        wakers: DoubleWaker {
            first: also,
            second: None,
        },
    }
}

pub(crate) struct AlsoWake<F, R>
where
    F: Future<Output = R>,
{
    fut: F,
    wakers: DoubleWaker,
}

impl<F, R> Unpin for AlsoWake<F, R> where F: Future<Output = R> {}

impl<F, R> AlsoWake<F, R>
where
    F: Future<Output = R>,
{
    pin_utils::unsafe_pinned!(fut: F);
}

impl<F, R> Future for AlsoWake<F, R>
where
    F: Future<Output = R>,
{
    type Output = R;
    fn poll(mut self: std::pin::Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        (&mut *self).wakers.set_second(lw);
        let local = self.wakers.local();
        self.fut().poll(&local)
    }
}

// FIXME this is a horror show of indirection

#[derive(Clone)]
struct DoubleWaker {
    first: Waker,
    second: Option<Waker>,
}

impl DoubleWaker {
    fn set_second(&mut self, second: &LocalWaker) {
        self.second.replace(second.clone().into());
    }

    fn local(&self) -> LocalWaker {
        // UNSAFE unclear why this is unsafe or whether we're upholding the invariants
        unsafe { std::task::local_waker(Arc::new(self.clone())) }
    }
}

impl std::task::Wake for DoubleWaker {
    fn wake(arc_self: &Arc<Self>) {
        arc_self.first.wake();
        if let Some(second) = &arc_self.second {
            second.wake();
        }
    }
}
