use crossbeam_channel::TryRecvError;
use memofs::{Vfs as MemoVfs, VfsEvent};
use std::ops::{Deref, DerefMut};
use tracing::{debug, info, trace};

pub struct Vfs {
    inner: MemoVfs,
}

impl Vfs {
    pub fn new() -> Self {
        Self { inner: MemoVfs::new_default() }
    }

    pub fn wait_for_changes(&self) {
        let changes = self.inner.event_receiver();
        match changes.recv().unwrap() {
            VfsEvent::Create(created) => info!(created = %created.display()),
            VfsEvent::Write(modified) => info!(modified = %modified.display()),
            VfsEvent::Remove(removed) => info!(removed = %removed.display()),
            _ => unimplemented!("unrecognized filesystem event"),
        }

        // TODO figure out how much memofs debounces, make sure its enough or we do some
        debug!("draining other fs events until quiescent");
        loop {
            match changes.try_recv() {
                Ok(event) => trace!(?event, "discarding"),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    unreachable!("other end is kept alive by ourselves")
                }
            }
        }
    }
}

impl Deref for Vfs {
    type Target = MemoVfs;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Vfs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
