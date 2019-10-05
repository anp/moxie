//! Embedding APIs offering finer-grained control over execution of the runtime.

use {
    crate::{sys, window, MemoElement},
    futures::task::{waker, ArcWake},
    moxie::{embed::Runtime, topo},
    std::{
        cell::{Cell, RefCell},
        rc::Rc,
        sync::Arc,
    },
    wasm_bindgen::{prelude::*, JsCast},
};

/// Wrapper around `moxie::embed::Runtime` which provides an `Env` for building trees of DOM nodes.
#[must_use]
pub struct WebRuntime(Runtime<Box<dyn FnMut()>, ()>);

impl WebRuntime {
    /// Construct a new `WebRuntime` which will maintain the children of the provided `parent`.
    ///
    /// On its own, a `WebRuntime` is inert and must either have its `run_once` method called when
    /// a re-render is needed, or be scheduled with [`WebRuntime::animation_frame_scheduler`].
    pub fn new(parent: sys::Element, mut root: impl FnMut() + 'static) -> Self {
        WebRuntime(Runtime::new(Box::new(move || {
            topo::call!(
                { root() },
                env! {
                    MemoElement => MemoElement::new(parent.clone()),
                }
            )
        })))
    }

    /// Run the root function in a fresh [moxie::Revision]. See [moxie::embed::Runtime::run_once]
    /// for details.
    pub fn run_once(&mut self) {
        self.0.run_once();
    }

    /// Pass ownership of this runtime to a "loop" which runs with `requestAnimationFrame`.
    pub fn animation_frame_scheduler(self) -> AnimationFrameScheduler {
        AnimationFrameScheduler(Rc::new(AnimationFrameState {
            wrt: RefCell::new(self),
            handle: Cell::new(None),
        }))
    }
}

/// Owns a `WebRuntime` and schedules its execution using `requestAnimationFrame`.
#[must_use]
pub struct AnimationFrameScheduler(Rc<AnimationFrameState>);

struct AnimationFrameState {
    wrt: RefCell<WebRuntime>,
    handle: Cell<Option<AnimationFrameHandle>>,
}

impl ArcWake for AnimationFrameScheduler {
    fn wake_by_ref(arc_self: &Arc<AnimationFrameScheduler>) {
        arc_self.ensure_scheduled(false);
    }
}

impl AnimationFrameScheduler {
    /// Consumes the scheduler to initiate a `requestAnimationFrame` callback loop where new
    /// animation frames are requested when state variables change. `WebRuntime::run_once` is called
    /// once per requested animation frame.
    pub fn run_on_state_changes(self) {
        let wrt2 = Rc::clone(&self.0);
        let waker = waker(Arc::new(self));
        {
            // ensure we've released our mutable borrow by running it in a separate block
            wrt2.wrt
                .borrow_mut()
                .0
                .set_state_change_waker(waker.clone());
        }
        waker.wake_by_ref();
    }

    /// Consumes the scheduler to initiate a `requestAnimationFrame` callback loop where new
    /// animation frames are requested immmediately after the last `moxie::Revision` is completed.
    /// `WebRuntime::run_once` is called once per requested animation frame.
    pub fn run_on_every_frame(self) {
        self.ensure_scheduled(true);
    }

    fn ensure_scheduled(&self, immediately_again: bool) {
        let existing = self.0.handle.replace(None);
        let handle = existing.unwrap_or_else(|| {
            let self2 = AnimationFrameScheduler(Rc::clone(&self.0));
            let callback = Closure::once(Box::new(move || {
                self2.0.handle.set(None);
                self2.0.wrt.borrow_mut().run_once();

                if immediately_again {
                    self2.ensure_scheduled(true);
                }
            }));

            AnimationFrameHandle::request(callback)
        });
        self.0.handle.set(Some(handle));
    }
}

// don't send these to workers until have a fix :P
unsafe impl Send for AnimationFrameScheduler {}
unsafe impl Sync for AnimationFrameScheduler {}

struct AnimationFrameHandle {
    raw: i32,
    /// Prefixed with an underscore because it is only read by JS, otherwise we'll get a warning.
    _callback: Closure<dyn FnMut()>,
}

impl AnimationFrameHandle {
    fn request(callback: Closure<dyn FnMut()>) -> Self {
        let raw = window()
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
        window().cancel_animation_frame(self.raw).ok();
    }
}
