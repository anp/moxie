//! Embedding APIs offering finer-grained control over execution of the runtime.

use {
    crate::{node::Node, MemoElement},
    moxie::{embed::Runtime, topo},
    raf::AnimationFrameScheduler,
    std::task::Waker,
};

#[cfg(feature = "rsdom")]
use {crate::node::rsdom, std::rc::Rc};

/// Wrapper around `moxie::embed::Runtime` which provides an `Env` for building trees of DOM nodes.
#[must_use]
pub struct WebRuntime(Runtime<Box<dyn FnMut()>, ()>);

impl WebRuntime {
    /// Construct a new `WebRuntime` which will maintain the children of the provided `parent`.
    ///
    /// On its own, a `WebRuntime` is inert and must either have its `run_once` method called when
    /// a re-render is needed, or be scheduled with [`WebRuntime::animation_frame_scheduler`].
    pub fn new(parent: impl Into<Node>, mut root: impl FnMut() + 'static) -> Self {
        let parent = parent.into();
        WebRuntime(Runtime::new(Box::new(move || {
            topo::call!(
                { root() },
                env! {
                    MemoElement => MemoElement::new(parent.clone()),
                }
            )
        })))
    }

    #[cfg(feature = "rsdom")]
    pub fn with_rsdom(root: impl FnMut() + 'static) -> (Self, Rc<rsdom::VirtNode>) {
        let container = rsdom::create_element("div");
        (WebRuntime::new(container.clone(), root), container)
    }

    /// Run the root function in a fresh [moxie::Revision]. See [moxie::embed::Runtime::run_once]
    /// for details.
    pub fn run_once(&mut self) {
        self.0.run_once();
    }

    /// Pass ownership of this runtime to a "loop" which runs with `requestAnimationFrame`.
    pub fn animation_frame_scheduler(self) -> AnimationFrameScheduler<Self> {
        AnimationFrameScheduler::new(self)
    }
}

impl raf::Tick for WebRuntime {
    fn tick(&mut self) {
        self.0.run_once();
    }
}

impl raf::Waking for WebRuntime {
    fn set_waker(&mut self, wk: Waker) {
        self.0.set_state_change_waker(wk);
    }
}
