//! Embedding APIs offering finer-grained control over execution of the runtime.

use crate::{cached_node::CachedNode, interfaces::node::Child};
use moxie::runtime::RunLoop;

/// Wrapper around `moxie::runtime::RunLoop` which provides an environment for
/// building trees of DOM nodes.
#[must_use]
pub struct WebRuntime {
    inner: RunLoop<Box<dyn FnMut()>>,
}

impl WebRuntime {
    /// Construct a new `WebRuntime` which will maintain the children of the
    /// provided `parent`.
    ///
    /// On its own, a `WebRuntime` is inert and must either have its `run_once`
    /// method called when a re-render is needed, or be scheduled with
    /// [`WebRuntime::animation_frame_scheduler`].
    pub fn new<Root: Child>(
        parent: impl Into<augdom::Node>,
        mut root: impl FnMut() -> Root + 'static,
    ) -> Self {
        let parent = parent.into();
        WebRuntime {
            inner: RunLoop::new(Box::new(move || {
                let parent = CachedNode::new(parent.clone());
                let new_root = topo::call(|| root());

                parent.ensure_child_attached(new_root.to_bind());
                parent.remove_trailing_children();
            })),
        }
    }

    /// Run the root function in a fresh `moxie::Revision`. See
    /// `moxie::runtime::RunLoop::run_once` for details.
    pub fn run_once(&mut self) {
        self.inner.run_once();
    }
}

#[cfg(feature = "webdom")]
mod web_impl {
    use super::*;
    use futures::{
        future::LocalFutureObj,
        task::{LocalSpawn, SpawnError},
    };

    impl WebRuntime {
        /// Create a new `div` and use that as the parent node for the runtime
        /// with which it is returned.
        pub fn in_web_div<Root: Child + 'static>(
            root: impl FnMut() -> Root + 'static,
        ) -> (Self, augdom::Node) {
            let container = augdom::document().create_element("div");
            let mut rt = WebRuntime::new(container.clone(), root);
            rt.inner.set_task_executor(WebSpawner);
            (rt, container)
        }

        /// Pass ownership of this runtime to a "loop" which runs with
        /// `requestAnimationFrame`.
        pub fn animation_frame_scheduler(self) -> raf::AnimationFrameScheduler<Self> {
            raf::AnimationFrameScheduler::new(self)
        }
    }

    impl raf::Tick for WebRuntime {
        fn tick(&mut self) {
            self.inner.run_once();
        }
    }

    impl raf::Waking for WebRuntime {
        fn set_waker(&mut self, wk: std::task::Waker) {
            self.inner.set_state_change_waker(wk);
        }
    }

    struct WebSpawner;

    impl LocalSpawn for WebSpawner {
        fn spawn_local_obj(&self, fut: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
            wasm_bindgen_futures::spawn_local(fut);
            Ok(())
        }
    }
}

#[cfg(feature = "rsdom")]
impl WebRuntime {
    /// Create a new virtual `div` and use that as the parent node for the
    /// runtime with which it is returned.
    pub fn in_rsdom_div<Root: Child>(
        mut root: impl FnMut() -> Root + 'static,
    ) -> (Self, augdom::Node) {
        use illicit::AsContext;
        let document = crate::raw::Document::new_virtual();
        let container = document.create_element("div");
        let root = move || document.clone().offer(&mut root);
        (WebRuntime::new(container.clone(), root), container)
    }
}
