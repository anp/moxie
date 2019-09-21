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

pub struct WebRuntime(Runtime<Box<dyn FnMut()>, ()>);

impl WebRuntime {
    pub fn new(new_parent: sys::Element, mut root: impl FnMut() + 'static) -> Self {
        WebRuntime(Runtime::new(Box::new(move || {
            topo::call!(
                { root() },
                env! {
                    MemoElement => MemoElement::new(&new_parent),
                }
            )
        })))
    }

    pub fn run_once(&mut self) {
        self.0.run_once();
    }

    pub fn animation_frame_scheduler(self) -> AnimationFrameScheduler {
        AnimationFrameScheduler(Rc::new(AnimationFrameState {
            wrt: RefCell::new(self),
            handle: Cell::new(None),
        }))
    }
}

pub struct AnimationFrameScheduler(Rc<AnimationFrameState>);

struct AnimationFrameState {
    wrt: RefCell<WebRuntime>,
    handle: Cell<Option<AnimationFrameHandle>>,
}

impl AnimationFrameScheduler {
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

    fn ensure_scheduled(&self) {
        let handle = self
            .0
            .handle
            .replace(None)
            .unwrap_or_else(|| AnimationFrameHandle::request(self.create_callback()));
        self.0.handle.set(Some(handle));
    }

    fn create_callback(&self) -> Closure<dyn FnMut()> {
        let state = Rc::clone(&self.0);
        Closure::once(Box::new(move || {
            state.handle.set(None);
            state.wrt.borrow_mut().run_once();
        }))
    }
}

impl ArcWake for AnimationFrameScheduler {
    fn wake_by_ref(arc_self: &Arc<AnimationFrameScheduler>) {
        arc_self.ensure_scheduled();
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
