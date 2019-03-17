use {
    futures::{
        stream::Stream,
        task::{Poll, Waker},
    },
    parking_lot::Mutex,
    std::{collections::VecDeque, pin::Pin, sync::Arc},
    time::Duration,
    timer::{Guard, Timer},
    webrender::api::RenderNotifier,
    winit::{EventsLoop, EventsLoopProxy},
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
    revision: u64,
    /// Responsible for polling the winit event loop.
    _timer: Timer,
    /// A handle to our waker timer.
    _guard: Guard,
}

unsafe impl Send for WindowEvents {}
unsafe impl Sync for WindowEvents {}

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
            revision: 0,
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

    fn poll_next(mut self: Pin<&mut Self>, wk: &Waker) -> Poll<Option<Self::Item>> {
        // refresh the waker we cache locally
        {
            let mut waker2 = Some(wk.clone());
            std::mem::swap(&mut *self.waker.lock(), &mut waker2);
        }

        if let Some(ev) = self.events.pop_front() {
            // if there are more events buffered, we'd like to deliver those immediately please
            if !self.events.is_empty() {
                wk.wake();
            }

            Poll::Ready(Some(ev))
        } else {
            let slf: &mut Self = &mut self;
            let (events_loop, events, revision) =
                (&mut slf.events_loop, &mut slf.events, &mut slf.revision);

            events_loop.poll_events(|inner| {
                *revision += 1;
                let ev = Event {
                    revision: *revision,
                    inner,
                };
                events.push_back(ev);
            });

            if let Some(ev) = self.events.pop_front() {
                if !self.events.is_empty() {
                    wk.wake();
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
        _render_time: Option<u64>,
    ) {
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

#[derive(Debug)]
pub struct Event {
    revision: u64,
    pub inner: winit::Event,
}
