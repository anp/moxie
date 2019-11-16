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

/// An executor based on [`futures::LocalPool`] which can run inside another executor. Its primary
/// usage here is to run handlers until stalled before & after each revision. Its goals limit the
/// executor to being useful for a small number of mostly quiet futures, typical of streams of
/// input events or other "per-frame" types of activity.
#[derive(Default)]
pub struct InBandExecutor {
    /// Futures currently being executed.
    pool: FuturesUnordered<LocalFutureObj<'static, ()>>,
    /// Futures which have not yet been polled.
    incoming: Rc<RefCell<Vec<LocalFutureObj<'static, ()>>>>,
}

/// Spawns futures onto an `InBandExecutor`.
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
