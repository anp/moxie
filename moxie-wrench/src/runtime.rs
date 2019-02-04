use {
    crate::{prelude::*, winit_future::WindowEvents},
    std::sync::atomic::{AtomicU64, Ordering},
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
                    self.compose.surface(scope!(), event.into());
                }
            }
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

pub struct Event {
    revision: Revision,
    inner: winit::Event,
}

pub struct WindowEvent {
    revision: Revision,
    window: WindowId,
    inner: winit::WindowEvent,
}
