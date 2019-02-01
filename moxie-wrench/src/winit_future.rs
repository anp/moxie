use futures::{
    stream::Stream,
    task::{LocalWaker, Poll, Waker},
};
use log::trace;
use parking_lot::Mutex;
use std::{collections::VecDeque, pin::Pin, sync::Arc};
use time::Duration;
use timer::{Guard, Timer};
use webrender::api::RenderNotifier;
use winit::{Event, EventsLoop, EventsLoopProxy};

/// This struct is an awful hack that adds unnecessary delays and buffering to the event system.
///
/// Just not very excited about writing all the winit-type stuff in async right now.
///
/// In addition to regularly polling the winit event loop, we also provide webrender a way to
/// notify the winit event loop that it's done rendering.
pub struct WindowEvents {
    events_loop: EventsLoop,
    events: VecDeque<Event>,
    waker: Arc<Mutex<Option<Waker>>>,
    /// Responsible for polling the winit event loop.
    _timer: Timer,
    /// A handle to our waker timer.
    _guard: Guard,
}

impl WindowEvents {
    pub fn new() -> Self {
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

    pub fn notifier(&self) -> Box<RenderNotifier> {
        let events_proxy = self.events_loop.create_proxy();
        Box::new(WindowNotifier {
            events_proxy,
            waker: self.waker.clone(),
        })
    }

    pub fn raw_loop(&self) -> &EventsLoop {
        &self.events_loop
    }
}

impl Stream for WindowEvents {
    type Item = winit::Event;

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

            events_loop.poll_events(|ev| events.push_back(ev));

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
