//! Provides a scheduled loop in the browser via `requestAnimationFrame`.

#![deny(missing_docs)]

use futures::task::{waker, ArcWake};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
    task::Waker,
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;

/// A value which can be mutably called by the scheduler.
pub trait Tick: 'static {
    /// Tick this value, indicating a new frame request is being fulfilled.
    fn tick(&mut self);
}

/// A value which can receive a waker from the scheduler that will request a new
/// frame when woken.
pub trait Waking {
    /// Receive a waker from the scheduler that calls `requestAnimationFrame`
    /// when woken.
    fn set_waker(&mut self, wk: Waker);
}

/// Owns a `WebRuntime` and schedules its execution using
/// `requestAnimationFrame`.
#[must_use]
pub struct AnimationFrameScheduler<Cb>(Rc<AnimationFrameState<Cb>>);

struct AnimationFrameState<Cb> {
    ticker: RefCell<Cb>,
    handle: Cell<Option<AnimationFrameHandle>>,
}

impl<T: Tick> ArcWake for AnimationFrameScheduler<T> {
    fn wake_by_ref(arc_self: &Arc<AnimationFrameScheduler<T>>) {
        arc_self.ensure_scheduled(false);
    }
}

impl<T: Tick> AnimationFrameScheduler<T> {
    /// Construct a new scheduler with the provided callback. `ticker.tick()`
    /// will be called once per fulfilled animation frame request.
    pub fn new(ticker: T) -> Self {
        AnimationFrameScheduler(Rc::new(AnimationFrameState {
            ticker: RefCell::new(ticker),
            handle: Cell::new(None),
        }))
    }

    fn ensure_scheduled(&self, immediately_again: bool) {
        let existing = self.0.handle.replace(None);
        let handle = existing.unwrap_or_else(|| {
            let self2 = AnimationFrameScheduler(Rc::clone(&self.0));
            let callback = Closure::once(Box::new(move || {
                self2.0.handle.set(None);

                self2.0.ticker.borrow_mut().tick();

                if immediately_again {
                    self2.ensure_scheduled(true);
                }
            }));

            AnimationFrameHandle::request(callback)
        });
        self.0.handle.set(Some(handle));
    }

    /// Consumes the scheduler to initiate a `requestAnimationFrame` callback
    /// loop where new animation frames are requested immmediately after the
    /// last `moxie::Revision` is completed. `WebRuntime::run_once` is
    /// called once per requested animation frame.
    pub fn run_on_every_frame(self) {
        self.ensure_scheduled(true);
    }
}

impl<T: Tick + Waking> AnimationFrameScheduler<T> {
    /// Consumes the scheduler to initiate a `requestAnimationFrame` callback
    /// loop where new animation frames are requested whenever the waker
    /// passed to the provided closure is woken.
    pub fn run_on_wake(self) {
        let state = Rc::clone(&self.0);
        let waker = waker(Arc::new(self));
        state.ticker.borrow_mut().set_waker(waker);
        state.ticker.borrow_mut().tick();
    }
}

// don't send these to workers until have a fix :P
unsafe impl<Cb> Send for AnimationFrameScheduler<Cb> {}
unsafe impl<Cb> Sync for AnimationFrameScheduler<Cb> {}

struct AnimationFrameHandle {
    raw: i32,
    /// Prefixed with an underscore because it is only read by JS, otherwise
    /// we'll get a warning.
    _callback: Closure<dyn FnMut()>,
}

impl AnimationFrameHandle {
    fn request(callback: Closure<dyn FnMut()>) -> Self {
        let raw =
            window().unwrap().request_animation_frame(callback.as_ref().unchecked_ref()).unwrap();

        Self { raw, _callback: callback }
    }
}

impl Drop for AnimationFrameHandle {
    fn drop(&mut self) {
        window().unwrap().cancel_animation_frame(self.raw).ok();
    }
}
