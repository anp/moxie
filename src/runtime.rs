use {
    crate::{prelude::*, winit_future::WindowEvents},
    std::{
        hash::{Hash, Hasher},
        sync::atomic::{AtomicU64, Ordering},
    },
    winit::WindowId,
};

#[derive(Default)]
pub struct RuntimeyWimey {
    compose: Composer,
    events: WindowEvents,
}

impl RuntimeyWimey {
    pub async fn gogo(self) {
        let mut revision = 0u64;
        while let Some(ev) = await!(self.events.next()) {
            match ev {
                winit::Event::WindowEvent { window_id, event } => {
                    self.compose.surface(
                        scope!(),
                        WindowEvent {
                            window: window_id,
                            inner: event,
                            revision: Revision::next(),
                        },
                    );
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(u64);

static current: AtomicU64 = AtomicU64::new(0);

impl Revision {
    pub fn current() -> Self {
        Self(current.load(Ordering::Relaxed))
    }

    fn next() -> Self {
        Self(current.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug)]
pub struct Event {
    revision: Revision,
    inner: winit::Event,
}

#[derive(Clone, Debug)]
pub struct WindowEvent {
    revision: Revision,
    window: WindowId,
    inner: winit::WindowEvent,
}

impl Hash for WindowEvent {
    fn hash<H: Hasher>(&self, h: &mut H) {
        unimplemented!()
    }
}

impl PartialEq for WindowEvent {
    fn eq(&self, other: &Self) -> bool {
        self.revision == other.revision && self.window == other.window
    }
}

impl Eq for WindowEvent {}
