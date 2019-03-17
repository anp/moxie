use {
    crate::{caps::CallsiteId, our_prelude::*},
    futures::{
        channel::mpsc::{self, Receiver},
        sink::SinkExt,
    },
    std::hash::{Hash, Hasher},
};

pub fn channel<T>(callsite: CallsiteId) -> (Sender<T>, Receiver<T>) {
    let (sender, receiver) = mpsc::channel(100);
    (
        Sender {
            source: callsite,
            inner: sender,
        },
        receiver,
    )
}

/// A channel sender that can be passed as an argument to components.
#[derive(Debug)]
pub struct Sender<T> {
    source: CallsiteId,
    inner: mpsc::Sender<T>,
}

impl<T> Sender<T> {
    pub async fn send(&mut self, t: T) {
        if await!(self.inner.send(t)).is_err() {
            warn!(
                "Stale event channel id {:?} received an event, ignoring.",
                self.source
            );
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender {
            source: self.source,
            inner: self.inner.clone(),
        }
    }
}

impl<T> PartialEq for Sender<T> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}
impl<T> Eq for Sender<T> {}

impl<T> Hash for Sender<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.source.hash(hasher);
    }
}
