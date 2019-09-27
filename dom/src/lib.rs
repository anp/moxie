#![warn(missing_docs)]

#[doc(hidden)]
pub use moxie::*;

use {
    crate::embed::WebRuntime,
    moxie,
    std::cell::Cell,
    tracing::*,
    wasm_bindgen::{prelude::*, JsCast},
};

pub mod elements;
pub mod embed;
pub mod prelude {
    pub use crate::{
        __element_impl, __text_impl, document, window, BlurEvent, ChangeEvent, ClickEvent,
        DoubleClickEvent, Event, KeyDownEvent,
    };
    pub use moxie::*;
    pub use wasm_bindgen::prelude::*;
}

#[proc_macro_hack::proc_macro_hack(support_nested)]
pub use moxml::moxml;
pub use web_sys as sys;

/// The "boot sequence" for a moxie-dom instance creates a [crate::embed::WebRuntime] with the
/// provided arguments and begins scheduling its execution with `requestAnimationFrame` on state
/// changes.
///
/// If you need to schedule your root function more or less frequently than when state variables are
/// updated, see the [embed](crate::embed) module for granular control over scheduling.
///
/// In terms of the embed module's APIs, this function constructs a new
/// [`WebRuntime`](crate::embed::WebRuntime) and begins scheduling it with an
/// [`AnimationFrameScheduler`](crate::embed::AnimationFrameScheduler) which requests an animation
/// frame only when there are updates to state variables.
pub fn boot(new_parent: impl AsRef<sys::Element> + 'static, root: impl FnMut() + 'static) {
    WebRuntime::new(new_parent.as_ref().to_owned(), root)
        .animation_frame_scheduler()
        .run_on_state_changes();
}

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

#[topo::aware]
#[topo::from_env(parent: MemoElement)]
pub fn text(s: impl ToString) {
    // TODO consider a ToOwned-based memoization API that's lower level?
    // memo_ref<Ref, Arg, Output>(reference: Ref, init: impl FnOnce(Arg) -> Output)
    // where Ref: ToOwned<Owned=Arg> + PartialEq, etcetcetc
    let text_node = memo!(s.to_string(), |s| document().create_text_node(s));
    parent.ensure_child_attached(&text_node);
}

#[topo::aware]
#[topo::from_env(parent: MemoElement)]
pub fn element<ChildRet>(ty: &'static str, with_elem: impl FnOnce(&MemoElement) -> ChildRet) {
    let elem = memo!(ty, |ty| document().create_element(ty).unwrap());
    parent.ensure_child_attached(&elem);
    let elem = MemoElement::new(elem);
    with_elem(&elem);
}

pub struct MemoElement {
    curr: Cell<Option<sys::Node>>,
    elem: sys::Element,
}

impl MemoElement {
    fn new(elem: sys::Element) -> Self {
        Self {
            curr: Cell::new(None),
            elem,
        }
    }

    /// Retrieves access to the raw HTML element underlying the `MemoElement`.
    ///
    /// Because this offers an escape hatch around the memoized mutations, it should be used with
    /// caution. Also because of this, it has a silly name intended to loudly announce that
    /// care must be taken.
    pub fn raw_element_that_has_sharp_edges_please_be_careful(&self) -> sys::Element {
        self.elem.clone()
    }

    // FIXME this should be topo-aware
    // TODO and it should be able to express its slot as an annotation
    pub fn attr(&self, name: &'static str, value: impl ToString) -> &Self {
        topo::call!(slot: name, {
            memo_with!(
                value.to_string(),
                |v| {
                    self.elem.set_attribute(name, v).unwrap();
                    scopeguard::guard(self.elem.clone(), move |elem| {
                        elem.remove_attribute(name).unwrap()
                    })
                },
                |_| {}
            )
        });
        self
    }

    // FIXME this should be topo-aware
    pub fn on<Ev>(&self, callback: impl FnMut(Ev) + 'static) -> &Self
    where
        Ev: 'static + Event,
    {
        topo::call!(slot: Ev::NAME, {
            memo_with!(
                moxie::embed::Revision::current(),
                |_| {
                    let target: &sys::EventTarget = self.elem.as_ref();
                    EventHandle::new(target.clone(), callback)
                },
                |_| {}
            );
        });
        self
    }

    fn ensure_child_attached(&self, new_child: &sys::Node) {
        let prev_sibling = self.curr.replace(Some(new_child.clone()));

        let existing = if prev_sibling.is_none() {
            self.elem.first_child()
        } else {
            prev_sibling.and_then(|p| p.next_sibling())
        };

        if let Some(existing) = existing {
            if !existing.is_same_node(Some(new_child)) {
                self.elem.replace_child(new_child, &existing).unwrap();
            }
        } else {
            self.elem.append_child(new_child).unwrap();
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
                MemoElement => MemoElement::new(self.elem.clone()),
            }
        );

        let mut next_to_remove = if let Some(c) = last_desired_child {
            c.next_sibling()
        } else {
            elem.first_child()
        };

        while let Some(to_remove) = next_to_remove {
            next_to_remove = to_remove.next_sibling();
            elem.remove_child(&to_remove).unwrap();
        }

        ret
    }
}

pub trait Event: AsRef<web_sys::Event> + JsCast {
    const NAME: &'static str;
}

struct Callback {
    cb: Closure<dyn FnMut(JsValue)>,
}

impl Callback {
    fn new<Ev>(mut cb: impl FnMut(Ev) + 'static) -> Self
    where
        Ev: Event,
    {
        let cb = Closure::wrap(Box::new(move |ev: JsValue| {
            let ev: Ev = ev.dyn_into().unwrap();
            cb(ev);
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
    fn new<Ev>(target: web_sys::EventTarget, callback: impl FnMut(Ev) + 'static) -> Self
    where
        Ev: Event,
    {
        let callback = Callback::new(callback);
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
