#![allow(missing_docs)]

#[doc(hidden)]
pub use moxie::*;

use {
    futures::task::{waker, ArcWake},
    moxie::{self, *},
    std::{
        cell::RefCell,
        rc::Rc,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    },
    tracing::*,
    wasm_bindgen::{prelude::*, JsCast},
    web_sys::Node as DomNode,
};

pub mod elements;
pub mod events;
pub mod prelude {
    pub use crate::{
        __mount_impl, // impl detail of topo leaking through here FIXME!
        document,
        elements::{element, Element},
        events::{
            BlurEvent, ChangeEvent, ClickEvent, DoubleClickEvent, Event, EventTarget, KeyDownEvent,
        },
        text,
        window,
    };
    pub use moxie::*;
}

pub use web_sys as sys;

pub fn window() -> web_sys::Window {
    web_sys::window().expect("must run from within a `window`")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("must run from withing a `window` with a valid `document`")
}

#[topo::bound]
pub fn mount(new_parent: impl Into<DomNode> + 'static, root: impl Component + Clone + 'static) {
    let new_parent = new_parent.into();
    let rt: Runtime<Box<dyn FnMut()>> = Runtime::new(Box::new(move || {
        produce_without_attaching!(MountedNode(new_parent.clone(), vec![]), || {
            show!(root.clone());
        });
    }));

    let wrt = WebRuntime { rt, handle: None };

    let wrt = Rc::new((AtomicBool::new(false), RefCell::new(wrt)));
    let wrt2 = Rc::clone(&wrt);

    let arc_waker = Arc::new(RuntimeWaker { wrt });
    let waker = waker(arc_waker);

    {
        // ensure we've released our mutable borrow by running it in a separate block
        wrt2.1.borrow_mut().rt.set_state_change_waker(waker.clone());
    }

    waker.wake_by_ref();
}

#[topo::bound]
pub fn produce_dom(
    node: impl Into<DomNode>,
    event_handles: Vec<events::EventHandle>,
    children: impl FnOnce(),
) {
    produce!(MountedNode(node.into(), event_handles), children);
}

struct MountedNode(DomNode, Vec<events::EventHandle>);

impl PartialEq for MountedNode {
    fn eq(&self, other: &Self) -> bool {
        self.0.is_same_node(Some(&other.0))
    }
}

#[derive(Debug)]
struct UnmountDomNodeOnDrop(DomNode);

impl Drop for UnmountDomNodeOnDrop {
    #[instrument]
    fn drop(&mut self) {
        if let Some(parent) = self.0.parent_node() {
            trace!("unmounting node from parent");
            let _dont_care = parent.remove_child(&self.0);
        }
    }
}

impl Node for MountedNode {
    type MountHandle = UnmountDomNodeOnDrop;
    fn child(
        &mut self,
        child_node: &MountedNode,
        mounted: Option<Self::MountHandle>,
    ) -> Self::MountHandle {
        if let Some(mounted) = mounted {
            if mounted.0.is_same_node(Some(&child_node.0)) {
                // trace!(target: "reusing dom node", { ?mounted });
                return mounted;
            }
        }

        self.0.append_child(&child_node.0).unwrap();
        UnmountDomNodeOnDrop(child_node.0.clone())
    }
}

struct WebRuntime {
    rt: Runtime<Box<dyn FnMut()>>,
    handle: Option<(i32, Closure<dyn FnMut()>)>,
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

            let closure = Closure::once(Box::new(move || {
                let scheduled = &wrt.0;
                let mut wrt = wrt.1.borrow_mut();
                wrt.handle = None;
                scopeguard::defer!(scheduled.store(false, Ordering::SeqCst));

                wrt.rt.run_once();
            }));
            let handle = window()
                .request_animation_frame(closure.as_ref().unchecked_ref())
                .unwrap();
            scheduled.store(true, Ordering::SeqCst);

            arc_self.wrt.1.borrow_mut().handle = Some((handle, closure));
        } else {
            trace!("skipped scheduling web runtime")
        }
    }
}
