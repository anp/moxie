//! Provides a scheduled loop in the browser via `requestAnimationFrame`.
#![warn(missing_docs)]

use {
    futures::task::{waker, ArcWake},
    std::{
        cell::{Cell, RefCell},
        rc::Rc,
        sync::Arc,
        task::Waker,
    },
    wasm_bindgen::{prelude::*, JsCast},
    web_sys::window,
};

pub trait Tick {
    fn tick(&mut self);
}

pub trait Waking {
    fn set_waker(&mut self, wk: Waker);
}

/// Owns a `WebRuntime` and schedules its execution using `requestAnimationFrame`.
#[must_use]
pub struct AnimationFrameScheduler<Cb>(Rc<AnimationFrameState<Cb>>);

struct AnimationFrameState<Cb> {
    ticker: RefCell<Cb>,
    handle: Cell<Option<AnimationFrameHandle>>,
}

impl<T: Tick + 'static> ArcWake for AnimationFrameScheduler<T> {
    fn wake_by_ref(arc_self: &Arc<AnimationFrameScheduler<T>>) {
        arc_self.ensure_scheduled(false);
    }
}

impl<T: Tick + 'static> AnimationFrameScheduler<T> {
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

    /// Consumes the scheduler to initiate a `requestAnimationFrame` callback loop where new
    /// animation frames are requested immmediately after the last `moxie::Revision` is completed.
    /// `WebRuntime::run_once` is called once per requested animation frame.
    pub fn run_on_every_frame(self) {
        self.ensure_scheduled(true);
    }
}

impl<T: Tick + Waking + 'static> AnimationFrameScheduler<T> {
    /// Consumes the scheduler to initiate a `requestAnimationFrame` callback loop where new
    /// animation frames are requested whenever the waker passed to the provided closure is woken.
    pub fn run_on_wake(self) {
        let state = Rc::clone(&self.0);
        let waker = waker(Arc::new(self));
        {
            // ensure we've released our mutable borrow by running it in a separate block
            state.ticker.borrow_mut().set_waker(waker.clone());
        }
        waker.wake_by_ref();
    }
}

// don't send these to workers until have a fix :P
unsafe impl<Cb> Send for AnimationFrameScheduler<Cb> {}
unsafe impl<Cb> Sync for AnimationFrameScheduler<Cb> {}

struct AnimationFrameHandle {
    raw: i32,
    /// Prefixed with an underscore because it is only read by JS, otherwise we'll get a warning.
    _callback: Closure<dyn FnMut()>,
}

impl AnimationFrameHandle {
    fn request(callback: Closure<dyn FnMut()>) -> Self {
        let raw = window()
            .unwrap()
            .request_animation_frame(callback.as_ref().unchecked_ref())
            .unwrap();

        Self {
            raw,
            _callback: callback,
        }
    }
}

impl Drop for AnimationFrameHandle {
    fn drop(&mut self) {
        window().unwrap().cancel_animation_frame(self.raw).ok();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
