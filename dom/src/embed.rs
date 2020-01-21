//! Embedding APIs offering finer-grained control over execution of the runtime.

use crate::MemoElement;
use augdom::Node;
use moxie::{embed::Runtime, prelude::topo};

/// Wrapper around `moxie::embed::Runtime` which provides an `Env` for building
/// trees of DOM nodes.
#[must_use]
pub struct WebRuntime(Runtime<Box<dyn FnMut(())>>);

impl WebRuntime {
    /// Construct a new `WebRuntime` which will maintain the children of the
    /// provided `parent`.
    ///
    /// On its own, a `WebRuntime` is inert and must either have its `run_once`
    /// method called when a re-render is needed, or be scheduled with
    /// [`WebRuntime::animation_frame_scheduler`].
    pub fn new(parent: impl Into<Node>, mut root: impl FnMut() + 'static) -> Self {
        let parent = parent.into();
        WebRuntime(Runtime::new(Box::new(move |()| {
            illicit::child_env!(MemoElement => MemoElement::new(parent.clone()))
                .enter(|| topo::call(|| root()))
        })))
    }

    /// Run the root function in a fresh `moxie::Revision`. See
    /// `moxie::embed::Runtime::run_once` for details.
    pub fn run_once(&mut self) {
        self.0.run_once(());
    }
}

#[cfg(feature = "webdom")]
impl WebRuntime {
    /// Create a new `div` and use that as the parent node for the runtime with
    /// which it is returned.
    pub fn in_web_div(root: impl FnMut() + 'static) -> (Self, augdom::sys::Element) {
        let container = augdom::document().create_element("div").unwrap();
        (WebRuntime::new(container.clone(), root), container)
    }

    /// Pass ownership of this runtime to a "loop" which runs with
    /// `requestAnimationFrame`.
    pub fn animation_frame_scheduler(self) -> raf::AnimationFrameScheduler<Self> {
        impl raf::Tick for WebRuntime {
            fn tick(&mut self) {
                self.0.run_once(());
            }
        }

        impl raf::Waking for WebRuntime {
            fn set_waker(&mut self, wk: std::task::Waker) {
                self.0.set_state_change_waker(wk);
            }
        }

        raf::AnimationFrameScheduler::new(self)
    }
}

#[cfg(feature = "rsdom")]
impl WebRuntime {
    /// Create a new virtual `div` and use that as the parent node for the
    /// runtime with which it is returned.
    pub fn in_rsdom_div(
        root: impl FnMut() + 'static,
    ) -> (Self, std::rc::Rc<augdom::rsdom::VirtNode>) {
        let container = augdom::rsdom::create_element("div");
        (WebRuntime::new(container.clone(), root), container)
    }
}
