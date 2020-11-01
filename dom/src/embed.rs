//! Embedding APIs offering finer-grained control over execution of the runtime.

use crate::{cached_node::CachedNode, interfaces::node::Child};
use futures::{
    future::LocalFutureObj,
    task::{LocalSpawn, SpawnError},
};
use moxie::runtime::RunLoop;

/// Wrapper around `moxie::runtime::RunLoop` and a root function which returns a
/// DOM node. After each call to `run_once` the node returned from the root
/// function is re-attached to the provided parent, removing other children.
#[must_use]
pub struct DomLoop {
    inner: RunLoop<Box<dyn FnMut()>>,
}

impl DomLoop {
    /// Construct a new `DomLoop` which will maintain the children of the
    /// provided `parent`.
    ///
    /// On its own, a `WebRuntime` is inert and must either have its `run_once`
    /// method called when a re-render is needed, or be scheduled with
    /// [`DomLoop::animation_frame_scheduler`].
    pub fn new<Root: Child + 'static>(
        parent: impl Into<augdom::Node>,
        mut root: impl (FnMut() -> Root) + 'static,
    ) -> Self {
        let parent = parent.into();

        let mut inner = RunLoop::new(Box::new(move || {
            let parent = CachedNode::new(parent.clone());
            let new_root = topo::call(|| root());

            parent.ensure_child_attached(new_root.to_bind());
            parent.remove_trailing_children();
        }) as Box<dyn FnMut()>);

        #[cfg(feature = "webdom")]
        {
            inner.set_task_executor(WebSpawner);
        }

        Self { inner }
    }

    /// Construct a new `DomLoop` which will maintain the children of a virtual
    /// `<div>`.
    ///
    /// Internally calls [`DomLoop::new`].
    #[cfg(feature = "rsdom")]
    pub fn new_virtual<Root: Child + 'static>(
        parent: impl Into<augdom::Node>,
        root: impl (FnMut() -> Root) + 'static,
    ) -> Self {
        Self::new(parent, augdom::in_virtual_document(root))
    }

    /// Run the root function in a fresh `moxie::Revision` and bind the returned
    /// node as the child of the loop's parent.
    pub fn run_once(&mut self) {
        self.inner.run_once();
    }
}

impl DomLoop {
    /// Pass ownership of this runtime to a "loop" which runs with
    /// `requestAnimationFrame`.
    pub fn animation_frame_scheduler(self) -> raf::AnimationFrameScheduler<Self> {
        raf::AnimationFrameScheduler::new(self)
    }
}

impl raf::Tick for DomLoop {
    fn tick(&mut self) {
        self.inner.run_once();
    }
}

impl raf::Waking for DomLoop {
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
