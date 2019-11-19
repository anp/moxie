//! An executor for futures that need to run with low latency relative to the runtime.

use {
    super::Spawner,
    futures::{
        future::{FutureObj, LocalFutureObj},
        stream::{FuturesUnordered, StreamExt},
        task::{LocalSpawn, Spawn, SpawnError},
    },
    std::{
        cell::RefCell,
        rc::{Rc, Weak},
        task::{Context, Poll, Waker},
    },
};

/// An executor styled after [`futures::LocalPool`] but which can run inside another executor. Its
/// primary usage here is to run handlers until stalled before & after each revision. Its goals
/// limit the executor to being useful for a small number of mostly quiet futures, typical of
/// streams of input events or other "per-frame" types of activity.
#[derive(Default)]
pub struct InBandExecutor {
    /// Futures currently being executed.
    pool: FuturesUnordered<LocalFutureObj<'static, ()>>,
    /// Futures which have not yet been polled.
    incoming: Rc<RefCell<Vec<LocalFutureObj<'static, ()>>>>,
}

/// Spawns futures onto an `InBandExecutor`.
#[derive(Clone)]
pub struct InBandSpawner(Weak<RefCell<Vec<LocalFutureObj<'static, ()>>>>);

impl InBandExecutor {
    /// Run the executor until it has stalled. This does not yet offer any timeout
    /// mechanism which works across platforms.
    pub fn run_until_stalled(&mut self, waker: &Waker) {
        let mut cx = Context::from_waker(waker);
        loop {
            // empty the incoming queue of newly-spawned tasks
            {
                let mut incoming = self.incoming.borrow_mut();
                for task in incoming.drain(..) {
                    self.pool.push(task)
                }
            }

            // try to execute the next ready future
            let ret = self.pool.poll_next_unpin(&mut cx);

            // check to see whether tasks were spawned by that
            if !self.incoming.borrow().is_empty() {
                continue;
            }

            // no queued tasks; we may be done
            match ret {
                Poll::Pending => return,
                Poll::Ready(None) => return,
                _ => {}
            }
        }
    }

    pub(crate) fn spawner(&self) -> Spawner {
        Spawner(Rc::new(InBandSpawner(Rc::downgrade(&self.incoming))))
    }

    #[cfg(test)]
    fn local_spawner(&self) -> InBandSpawner {
        InBandSpawner(Rc::downgrade(&self.incoming))
    }
}

impl Spawn for InBandSpawner {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        self.spawn_local_obj(future.into())
    }

    fn status(&self) -> Result<(), SpawnError> {
        self.status_local()
    }
}

impl LocalSpawn for InBandSpawner {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        if let Some(incoming) = self.0.upgrade() {
            incoming.borrow_mut().push(future);
            Ok(())
        } else {
            Err(SpawnError::shutdown())
        }
    }

    fn status_local(&self) -> Result<(), SpawnError> {
        if self.0.upgrade().is_some() {
            Ok(())
        } else {
            Err(SpawnError::shutdown())
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        futures::{
            future::{lazy, poll_fn, Future},
            task::{noop_waker, Context, LocalSpawn, Poll},
        },
        std::{
            cell::{Cell, RefCell},
            pin::Pin,
            rc::Rc,
        },
    };

    struct Pending(Rc<()>);

    impl Future for Pending {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
            Poll::Pending
        }
    }

    fn pending() -> Pending {
        Pending(Rc::new(()))
    }

    #[test]
    fn run_until_stalled_returns_if_empty() {
        let mut pool = InBandExecutor::default();
        let wk = noop_waker();
        pool.run_until_stalled(&wk);
        pool.run_until_stalled(&wk);
    }

    #[test]
    fn run_until_stalled_returns_multiple_times() {
        let mut pool = InBandExecutor::default();
        let wk = noop_waker();
        let spawn = pool.local_spawner();
        let cnt = Rc::new(Cell::new(0));

        let cnt1 = cnt.clone();
        spawn
            .spawn_local_obj(Box::pin(lazy(move |_| cnt1.set(cnt1.get() + 1))).into())
            .unwrap();
        pool.run_until_stalled(&wk);
        assert_eq!(cnt.get(), 1);

        let cnt2 = cnt.clone();
        spawn
            .spawn_local_obj(Box::pin(lazy(move |_| cnt2.set(cnt2.get() + 1))).into())
            .unwrap();
        pool.run_until_stalled(&wk);
        assert_eq!(cnt.get(), 2);
    }

    #[test]
    fn run_until_stalled_runs_spawned_sub_futures() {
        let mut pool = InBandExecutor::default();
        let wk = noop_waker();
        let spawn = pool.local_spawner();
        let cnt = Rc::new(Cell::new(0));

        let inner_spawner = spawn.clone();
        let cnt1 = cnt.clone();
        spawn
            .spawn_local_obj(
                Box::pin(poll_fn(move |_| {
                    cnt1.set(cnt1.get() + 1);

                    let cnt2 = cnt1.clone();
                    inner_spawner
                        .spawn_local_obj(Box::pin(lazy(move |_| cnt2.set(cnt2.get() + 1))).into())
                        .unwrap();

                    Poll::Pending
                }))
                .into(),
            )
            .unwrap();

        pool.run_until_stalled(&wk);
        assert_eq!(cnt.get(), 2);
    }

    #[test]
    fn run_until_stalled_executes_all_ready() {
        const ITER: usize = 200;
        const PER_ITER: usize = 3;

        let cnt = Rc::new(Cell::new(0));

        let mut pool = InBandExecutor::default();
        let wk = noop_waker();
        let spawn = pool.local_spawner();

        for i in 0..ITER {
            for _ in 0..PER_ITER {
                spawn.spawn_local_obj(Box::pin(pending()).into()).unwrap();

                let cnt = cnt.clone();
                spawn
                    .spawn_local_obj(
                        Box::pin(lazy(move |_| {
                            cnt.set(cnt.get() + 1);
                            ()
                        }))
                        .into(),
                    )
                    .unwrap();

                // also add some pending tasks to test if they are ignored
                spawn.spawn_local_obj(Box::pin(pending()).into()).unwrap();
            }
            assert_eq!(cnt.get(), i * PER_ITER);
            pool.run_until_stalled(&wk);
            assert_eq!(cnt.get(), (i + 1) * PER_ITER);
        }
    }

    #[test]
    fn tasks_are_scheduled_fairly() {
        let state = Rc::new(RefCell::new([0, 0]));

        struct Spin {
            state: Rc<RefCell<[i32; 2]>>,
            idx: usize,
        }

        impl Future for Spin {
            type Output = ();

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                let mut state = self.state.borrow_mut();

                if self.idx == 0 {
                    let diff = state[0] - state[1];

                    assert!(diff.abs() <= 1);

                    if state[0] >= 50 {
                        return Poll::Ready(());
                    }
                }

                state[self.idx] += 1;

                if state[self.idx] >= 100 {
                    return Poll::Ready(());
                }

                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }

        let mut pool = InBandExecutor::default();
        let wk = noop_waker();
        let spawn = pool.local_spawner();

        spawn
            .spawn_local_obj(
                Box::pin(Spin {
                    state: state.clone(),
                    idx: 0,
                })
                .into(),
            )
            .unwrap();

        spawn
            .spawn_local_obj(Box::pin(Spin { state, idx: 1 }).into())
            .unwrap();

        pool.run_until_stalled(&wk);
    }
}
