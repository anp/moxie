#![allow(missing_docs)]

#[doc(hidden)]
pub use moxie::*;

use {
    futures::task::{waker, ArcWake},
    moxie::{self, *},
    std::{
        cell::RefCell,
        collections::HashMap,
        fmt::{Debug, Formatter, Result as FmtResult},
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

pub mod prelude {
    pub use crate::{
        document, window, BlurEvent, ChangeEvent, ClickEvent, DoubleClickEvent, Event, KeyDownEvent,
    };
    pub use moxie::*;
    pub use wasm_bindgen::prelude::*;
}

pub use web_sys as sys;

pub fn window() -> web_sys::Window {
    web_sys::window().expect("must run from within a `window`")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("must run from within a `window` with a valid `document`")
}

pub fn run_with_parent(new_parent: impl Into<DomNode> + 'static, mut root: impl FnMut() + 'static) {
    let new_parent = new_parent.into();
    let rt: Runtime<Box<dyn FnMut()>> = Runtime::new(Box::new(move || {
        // FIXME pass the new_parent as the parent
        root();
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

struct MountedNode(DomNode, Vec<EventHandle>);

struct UnmountDomNodeOnDrop(DomNode);

impl Drop for UnmountDomNodeOnDrop {
    fn drop(&mut self) {
        if let Some(parent) = self.0.parent_node() {
            trace!("unmounting node from parent");
            let _dont_care = parent.remove_child(&self.0);
        }
    }
}

// impl Node for MountedNode {
//     type MountHandle = UnmountDomNodeOnDrop;
//     fn child(&mut self, child_node: &MountedNode) -> Self::MountHandle {
//         self.0.append_child(&child_node.0).unwrap();
//         UnmountDomNodeOnDrop(child_node.0.clone())
//     }
// }

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

#[topo::bound]
pub fn text(s: &str) {
    // let text_node = memo!(self.0, |text| document().create_text_node(text));
    unimplemented!()
}

pub struct MemoElement(sys::Element);

impl MemoElement {
    pub fn attr(self, name: &str, value: &str) -> Self {
        // TODO make sure these undo themselves if not called in a revision
        topo::call!(slot: name, {
            memo!(value.to_string(), |value| self
                .0
                .set_attribute(name, value)
                .unwrap());
        });
        self
    }

    pub fn on<Ev, State, Updater>(mut self, updater: Updater, key: Key<State>) -> Self
    where
        Ev: 'static + Event,
        State: 'static,
        Updater: 'static + FnMut(Ev, &State) -> Option<State>,
    {
        // TODO add the event handler to this type
        self
    }

    pub fn inner<Ret>(self, children: impl FnOnce() -> Ret) -> Ret {
        topo::call!(
            { children() },
            env! {
                MemoElement => MemoElement(self.0.clone()),
            }
        )
    }
}

#[topo::bound]
pub fn element(ty: &str) -> MemoElement {
    unimplemented!()
}

pub trait Event: AsRef<web_sys::Event> + JsCast {
    const NAME: &'static str;
}

struct Callback {
    cb: Closure<dyn FnMut(JsValue)>,
}

impl Callback {
    fn new<Ev, State, Updater>(key: Key<State>, mut updater: Updater) -> Self
    where
        Ev: Event,
        State: 'static,
        Updater: FnMut(Ev, &State) -> Option<State> + 'static,
    {
        let cb = Closure::wrap(Box::new(move |ev: JsValue| {
            let ev: Ev = ev.dyn_into().unwrap();
            key.update(|prev| updater(ev, prev));
        }) as Box<dyn FnMut(JsValue)>);
        Self { cb }
    }

    fn as_fn(&self) -> &js_sys::Function {
        self.cb.as_ref().unchecked_ref()
    }
}

#[must_use]
pub struct EventHandle {
    target: web_sys::EventTarget,
    callback: Callback,
    name: &'static str,
}

impl EventHandle {
    fn new<Ev, State, Updater>(
        target: web_sys::EventTarget,
        key: Key<State>,
        updater: Updater,
    ) -> Self
    where
        Ev: Event,
        State: 'static,
        Updater: FnMut(Ev, &State) -> Option<State> + 'static,
    {
        let callback = Callback::new(key, updater);
        let name = Ev::NAME;
        target
            .add_event_listener_with_callback(name, callback.as_fn())
            .unwrap();
        Self {
            target,
            callback,
            name,
        }
    }
}

impl Drop for EventHandle {
    fn drop(&mut self) {
        self.target
            .remove_event_listener_with_callback(self.name, self.callback.as_fn())
            .unwrap();
    }
}

macro_rules! event_ty {
    ($name:ident, $ty_str:expr, $parent_ty:ty) => {
        #[wasm_bindgen]
        pub struct $name($parent_ty);

        impl AsRef<web_sys::Event> for $name {
            fn as_ref(&self) -> &web_sys::Event {
                self.0.as_ref()
            }
        }

        impl AsRef<JsValue> for $name {
            fn as_ref(&self) -> &JsValue {
                self.0.as_ref()
            }
        }

        impl std::ops::Deref for $name {
            type Target = $parent_ty;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl JsCast for $name {
            fn instanceof(val: &JsValue) -> bool {
                <$parent_ty as JsCast>::instanceof(val)
            }

            fn unchecked_from_js(val: JsValue) -> Self {
                $name(<$parent_ty as JsCast>::unchecked_from_js(val))
            }

            fn unchecked_from_js_ref(_val: &JsValue) -> &Self {
                unimplemented!()
            }
        }

        impl Event for $name {
            const NAME: &'static str = $ty_str;
        }
    };
}

event_ty!(BlurEvent, "blur", web_sys::FocusEvent);
event_ty!(ChangeEvent, "change", web_sys::Event);
event_ty!(ClickEvent, "click", web_sys::MouseEvent);
event_ty!(DoubleClickEvent, "dblclick", web_sys::MouseEvent);
event_ty!(KeyDownEvent, "keydown", web_sys::KeyboardEvent);

#[cfg(test)]
pub mod tests {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn hello_world() {
        println!("look ma");
    }
}
