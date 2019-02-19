use {
    crate::prelude::*,
    futures::{
        stream::Stream,
        task::{LocalWaker, Poll, Waker},
    },
    parking_lot::Mutex,
    std::{
        collections::VecDeque,
        pin::Pin,
        sync::atomic::{AtomicU64, Ordering},
        sync::Arc,
    },
    time::Duration,
    timer::{Guard, Timer},
    webrender::api::RenderNotifier,
    winit::{EventsLoop, EventsLoopProxy, WindowId},
};

/// Wraps a winit event loop to provide a `futures::stream::Stream`.
///
/// This struct is an awful hack that adds unnecessary delays and buffering to the event system.
///
/// Just not very excited about writing all the winit-type stuff in async right now.
///
/// In addition to regularly polling the winit event loop, we also provide webrender a way to
/// notify the winit event loop that it's done rendering.
pub(crate) struct WindowEvents {
    events_loop: EventsLoop,
    events: VecDeque<Event>,
    waker: Arc<Mutex<Option<Waker>>>,
    /// Responsible for polling the winit event loop.
    _timer: Timer,
    /// A handle to our waker timer.
    _guard: Guard,
}

impl WindowEvents {
    pub(crate) fn new() -> Self {
        let waker = Arc::new(Mutex::new(None));
        let remote = waker.clone();

        let events_loop = EventsLoop::new();
        let _timer = Timer::new();
        let _guard =
            _timer.schedule_repeating(Duration::milliseconds(1), move || try_wake(&remote));

        WindowEvents {
            events_loop,
            events: VecDeque::new(),
            waker,
            _timer,
            _guard,
        }
    }

    pub(crate) fn notifier(&self) -> Box<RenderNotifier> {
        let events_proxy = self.events_loop.create_proxy();
        Box::new(WindowNotifier {
            events_proxy,
            waker: self.waker.clone(),
        })
    }

    pub(crate) fn raw_loop(&self) -> &EventsLoop {
        &self.events_loop
    }
}

impl Default for WindowEvents {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for WindowEvents {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Option<Self::Item>> {
        // refresh the waker we cache locally
        {
            let mut waker2 = Some(lw.clone().into_waker());
            std::mem::swap(&mut *self.waker.lock(), &mut waker2);
        }

        if let Some(ev) = self.events.pop_front() {
            // if there are more events buffered, we'd like to deliver those immediately please
            if !self.events.is_empty() {
                lw.wake();
            }

            Poll::Ready(Some(ev))
        } else {
            let slf: &mut Self = &mut self;
            let (events_loop, events) = (&mut slf.events_loop, &mut slf.events);

            events_loop.poll_events(|ev| {
                let ev = Event {
                    revision: Revision::next(),
                    inner: ev,
                };
                events.push_back(ev);
            });

            if let Some(ev) = self.events.pop_front() {
                if !self.events.is_empty() {
                    lw.wake();
                }
                Poll::Ready(Some(ev))
            } else {
                Poll::Pending
            }
        }
    }
}

struct WindowNotifier {
    events_proxy: EventsLoopProxy,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl RenderNotifier for WindowNotifier {
    fn clone(&self) -> Box<RenderNotifier> {
        Box::new(WindowNotifier {
            events_proxy: self.events_proxy.clone(),
            waker: self.waker.clone(),
        })
    }

    fn wake_up(&self) {
        self.events_proxy.wakeup().unwrap();
        try_wake(&self.waker);
    }

    fn new_frame_ready(
        &self,
        _: webrender::api::DocumentId,
        _scrolled: bool,
        _composite_needed: bool,
        render_time: Option<u64>,
    ) {
        trace!("New frame is ready. render time: {:?}", render_time);
        self.wake_up();
    }
}

fn try_wake(waker: &Arc<Mutex<Option<Waker>>>) {
    let waker = waker.lock();
    let waker: Option<&Waker> = waker.as_ref();
    if let Some(waker) = waker {
        waker.wake();
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(u64);

static CURRENT_REVISION: AtomicU64 = AtomicU64::new(0);

impl Revision {
    // /// Get the current revision, or "tick". Every event recieved advances the revision by 1, so
    // /// not all revisions will cause a recomposition to be executed, and so an even smaller number
    // /// will cause a new frame to be generated.
    // pub fn current() -> Self {
    //     Self(CURRENT_REVISION.load(Ordering::Relaxed))
    // }

    /// Get the next revision, advancing the global revision counter by 1.
    ///
    /// Note: this is private because user-defined code should be able to percieve the passage of
    /// time, but only the event system should be able to drive it forward.
    fn next() -> Self {
        Self(CURRENT_REVISION.fetch_add(1, Ordering::Relaxed))
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

impl PartialEq for WindowEvent {
    fn eq(&self, other: &Self) -> bool {
        self.revision == other.revision && self.window == other.window
    }
}

impl Eq for WindowEvent {}
