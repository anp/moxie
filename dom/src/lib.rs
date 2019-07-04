#![warn(missing_docs)]

pub use moxie::*;

use {
    futures::task::ArcWake,
    moxie::{self, *},
    std::{cell::RefCell, fmt::Debug, rc::Rc, sync::Arc},
    stdweb::{traits::*, *},
    tokio_trace::*,
};

#[topo::bound]
pub fn mount(new_parent: web::Node, root: impl Component + Clone + Debug + PartialEq + 'static) {
    let rt: Runtime<Box<dyn FnMut()>> = Runtime::new(Box::new(move || {
        moxie::produce_root!(MountedNode(new_parent.clone()), || {
            show!(root.clone());
        });
    }));

    let wrt = WebRuntime { rt, handle: None };

    let mut wrt = Rc::new(RefCell::new(wrt));
    let wrt2 = Rc::clone(&wrt);

    let waker = ArcWake::into_waker(Arc::new(RuntimeWaker { wrt }));

    wrt2.borrow_mut()
        .rt
        .set_state_change_waker(waker)
        .run_once();
}

#[topo::bound]
pub fn produce_dom(node: impl web::INode, children: impl FnOnce()) {
    produce!(MountedNode(node.as_node().to_owned()), children);
}

struct MountedNode(web::Node);

impl Node for MountedNode {
    fn child(&mut self, id: topo::Id, child_node: &MountedNode) {
        self.0.append_child(&child_node.0);
    }
}

struct WebRuntime {
    rt: Runtime<Box<dyn FnMut()>>,
    handle: Option<web::RequestAnimationFrameHandle>,
}

struct RuntimeWaker {
    wrt: Rc<RefCell<WebRuntime>>,
}

impl ArcWake for RuntimeWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let needs_handle = { arc_self.wrt.borrow().handle.is_none() };

        if needs_handle {
            trace!("wake web runtime");
            let wrt = Rc::clone(&arc_self.wrt);

            let handle = web::window().request_animation_frame(move |_time| {
                let mut wrt = wrt.borrow_mut();
                wrt.handle = None;
                wrt.rt.run_once();
            });

            arc_self.wrt.borrow_mut().handle = Some(handle);
        } else {
            trace!("")
        }
    }
}
