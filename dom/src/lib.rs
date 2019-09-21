//! ## Lifecycle
//!
//! ```text
//!                    +------+
//!                    | boot |
//!                    +---+--+
//!                        |
//!              +---------v------+
//!              | calling root() |
//!              +--+---------^---+
//!                 |         |
//!  +--------------v----+    | next
//!  | awaiting state 🛆 |    | frame
//!  +------------+------+    |
//! event occurs, |           |
//! updates state |           |
//!       +-------v-----------+---+
//!       | requestAnimationFrame |
//!       +-----------------------+
//! ```
//!
#![warn(missing_docs)]

#[doc(hidden)]
pub use moxie::*;

use {
    futures::task::{waker, ArcWake},
    moxie,
    std::{
        cell::{Cell, RefCell},
        rc::Rc,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    },
    tracing::*,
    wasm_bindgen::{prelude::*, JsCast},
};

pub mod prelude {
    pub use crate::{
        __element_impl, __text_impl, document, window, BlurEvent, ChangeEvent, ClickEvent,
        DoubleClickEvent, Event, KeyDownEvent,
    };
    pub use moxie::*;
    pub use wasm_bindgen::prelude::*;
}

pub use web_sys as sys;

/// Returns the current window. Panics if no window is available.
pub fn window() -> sys::Window {
    sys::window().expect("must run from within a `window`")
}

/// Returns the current document. Panics if called outside a web document context.
pub fn document() -> sys::Document {
    window()
        .document()
        .expect("must run from within a `window` with a valid `document`")
}

/// The "boot sequence" for a moxie-dom instance creates a [moxie::embed::Runtime] with the
/// provided arguments and begins scheduling its execution.
///
/// The instance created here is scoped to `new_parent` and assumes that it "owns" the mutation
/// of `new_parent`'s children.
///
/// TODO make this either event driven or browser-driven?
pub fn boot(new_parent: impl AsRef<sys::Element> + 'static, mut root: impl FnMut() + 'static) {
    let new_parent = new_parent.as_ref().to_owned();
    let rt: embed::Runtime<Box<dyn FnMut()>, ()> = embed::Runtime::new(Box::new(move || {
        topo::call!(
            { root() },
            env! {
                MemoElement => MemoElement::new(&new_parent),
            }
        )
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

struct WebRuntime {
    rt: embed::Runtime<Box<dyn FnMut()>, ()>,
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
            trace!("web runtime already scheduled");
        }
    }
}

#[topo::aware]
#[topo::from_env(parent: MemoElement)]
pub fn text(s: impl ToString) {
    // TODO consider a ToOwned-based memoization API that's lower level?
    // memo_ref<Ref, Arg, Output>(reference: Ref, init: impl FnOnce(Arg) -> Output)
    // where Ref: ToOwned<Owned=Arg> + PartialEq, etcetcetc
    let text_node = document().create_text_node(&s.to_string());
    parent.ensure_child_attached(&text_node);
}

#[topo::aware]
#[topo::from_env(parent: MemoElement)]
pub fn element<ChildRet>(ty: &str, with_elem: impl FnOnce(&MemoElement) -> ChildRet) {
    let elem = document().create_element(ty).unwrap();
    parent.ensure_child_attached(&elem);
    let elem = MemoElement {
        elem,
        curr: Cell::new(None),
    };
    with_elem(&elem);
}

pub struct MemoElement {
    curr: Cell<Option<sys::Node>>,
    elem: sys::Element,
}

impl MemoElement {
    fn new(elem: &sys::Element) -> Self {
        Self {
            curr: Cell::new(elem.first_child()),
            elem: elem.clone(),
        }
    }

    // FIXME this should be topo-aware
    // TODO and it should be able to express its slot as an annotation
    pub fn attr(&self, name: &str, value: impl ToString) -> &Self {
        self.elem.set_attribute(name, &value.to_string()).unwrap();
        self
    }

    // FIXME this should be topo-aware
    pub fn on<Ev, State, Updater>(&self, updater: Updater, key: Key<State>) -> &Self
    where
        Ev: 'static + Event,
        State: 'static,
        Updater: 'static + FnMut(Ev, &State) -> Option<State>,
    {
        topo::call!(slot: Ev::NAME, {
            memo_with!(
                embed::Revision::current(),
                |_| {
                    let target: &sys::EventTarget = self.elem.as_ref();
                    EventHandle::new(target.clone(), key, updater)
                },
                |_| {}
            );
        });
        self
    }

    fn ensure_child_attached(&self, node: &sys::Node) {
        let prev = self.curr.replace(Some(node.clone()));

        if let Some(curr) = prev.and_then(|p| p.next_sibling()) {
            if !curr.is_same_node(Some(node)) {
                self.elem.replace_child(node, &curr).unwrap();
            }
        } else {
            self.elem.append_child(node).unwrap();
        }
    }

    // FIXME this should be topo-aware
    pub fn inner<Ret>(&self, children: impl FnOnce() -> Ret) -> Ret {
        let elem = self.elem.clone();
        let last_desired_child;
        let ret;
        topo::call!(
            {
                ret = children();

                // before this melement is dropped when the environment goes out of scope,
                // we need to get the last recorded child from this revision
                last_desired_child = topo::Env::expect::<MemoElement>().curr.replace(None);
            },
            env! {
                MemoElement => self.cloned(),
            }
        );

        let mut next_to_remove = last_desired_child.and_then(|c| c.next_sibling());
        while let Some(to_remove) = next_to_remove {
            next_to_remove = to_remove.next_sibling();
            elem.remove_child(&to_remove).unwrap();
        }

        ret
    }

    fn cloned(&self) -> Self {
        let curr = self.curr.replace(None);
        self.curr.replace(curr.clone());
        Self {
            elem: self.elem.clone(),
            curr: Cell::new(curr),
        }
    }
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
