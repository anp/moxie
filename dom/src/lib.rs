#![warn(missing_docs)]

#[doc(hidden)]
pub use moxie::*;

use {
    futures::task::ArcWake,
    moxie::{self, *},
    std::{
        cell::RefCell,
        fmt::Debug,
        rc::Rc,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    },
    stdweb::{traits::*, *},
    tracing::*,
};

pub mod elements;
pub mod events;

#[topo::bound]
pub fn mount(
    new_parent: impl web::INode + 'static,
    root: impl Component + Clone + Debug + PartialEq + 'static,
) {
    let rt: Runtime<Box<dyn FnMut()>> = Runtime::new(Box::new(move || {
        produce_without_attaching!(MountedNode(new_parent.as_node().to_owned()), || {
            show!(root.clone());
        });
    }));

    let wrt = WebRuntime { rt, handle: None };

    let wrt = Rc::new((AtomicBool::new(false), RefCell::new(wrt)));
    let wrt2 = Rc::clone(&wrt);

    let waker = ArcWake::into_waker(Arc::new(RuntimeWaker { wrt }));

    {
        // ensure we've released our mutable borrow by running it in a separate block
        wrt2.1.borrow_mut().rt.set_state_change_waker(waker.clone());
    }

    waker.wake_by_ref();
}

#[topo::bound]
pub fn produce_dom(node: impl web::INode, children: impl FnOnce()) {
    produce!(MountedNode(node.as_node().to_owned()), children);
}

struct MountedNode(web::Node);

struct UnmountDomNodeOnDrop(web::Node);

impl Drop for UnmountDomNodeOnDrop {
    fn drop(&mut self) {
        if let Some(parent) = self.0.parent_node() {
            trace!("unmounting node from parent");
            let _dont_care = parent.remove_child(&self.0);
        }
    }
}

impl Node for MountedNode {
    type MountHandle = UnmountDomNodeOnDrop;
    fn child(&mut self, child_node: &MountedNode) -> Self::MountHandle {
        self.0.append_child(&child_node.0);
        UnmountDomNodeOnDrop(child_node.0.clone())
    }
}

struct WebRuntime {
    rt: Runtime<Box<dyn FnMut()>>,
    handle: Option<web::RequestAnimationFrameHandle>,
}

struct RuntimeWaker {
    wrt: Rc<(AtomicBool, RefCell<WebRuntime>)>,
}

// don't send these to workers until have a fix :P
unsafe impl Send for RuntimeWaker {}
unsafe impl Sync for RuntimeWaker {}

impl ArcWake for RuntimeWaker {
    fn wake_by_ref(arc_self: &Arc<RuntimeWaker>) {
        let scheduled: &AtomicBool = &arc_self.wrt.0;
        if !scheduled.load(Ordering::SeqCst) {
            trace!("wake web runtime, scheduling");
            let wrt = Rc::clone(&arc_self.wrt);

            scheduled.store(true, Ordering::SeqCst);
            let handle = web::window().request_animation_frame(move |_time| {
                let scheduled = &wrt.0;
                let mut wrt = wrt.1.borrow_mut();
                wrt.handle = None;
                scopeguard::defer!(scheduled.store(false, Ordering::SeqCst));

                wrt.rt.run_once();
            });

            arc_self.wrt.1.borrow_mut().handle = Some(handle);
        } else {
            trace!("skipped scheduling web runtime")
        }
    }
}
