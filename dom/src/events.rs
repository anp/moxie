use {
    crate::*,
    std::fmt::{Debug, Formatter, Result as FmtResult},
};

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

pub trait EventTarget: Sized {
    fn handlers(&mut self) -> &mut Handlers;

    fn on<Ev, State, Updater>(mut self, updater: Updater, key: Key<State>) -> Self
    where
        Ev: 'static + Event,
        State: 'static,
        Updater: 'static + FnMut(Ev, &State) -> Option<State>,
    {
        self.handlers().add_listener(key, updater);
        self
    }
}

#[derive(Default)]
pub struct Handlers {
    inner: Vec<Box<dyn FnOnce(&web_sys::EventTarget) -> EventHandle + 'static>>,
}

impl Handlers {
    fn add_listener<Ev, State, Updater>(&mut self, key: Key<State>, updater: Updater)
    where
        Ev: Event,
        State: 'static,
        Updater: 'static + FnMut(Ev, &State) -> Option<State>,
    {
        self.inner.push(Box::new(move |target| -> EventHandle {
            EventHandle::new(target.to_owned(), key, updater)
        }));
    }

    pub(crate) fn apply(self, target: &web_sys::EventTarget) -> Vec<EventHandle> {
        self.inner
            .into_iter()
            .map(|handler| handler(target))
            .collect()
    }
}

impl Debug for Handlers {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Handlers").finish()
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
