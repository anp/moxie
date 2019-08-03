use {
    crate::*,
    std::fmt::{Debug, Formatter, Result as FmtResult},
};

pub trait Event: AsRef<web_sys::Event> {
    const NAME: &'static str;
}

#[must_use]
pub struct EventHandle {
    target: web_sys::EventTarget,
    callback: Closure<dyn FnMut()>,
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
        Updater: FnOnce(&State, Ev) -> Option<State>,
    {
        let callback =
            Closure::wrap(Box::new(move || info!("callback called")) as Box<dyn FnMut()>);
        debug!("binding event listener");
        let name = Ev::NAME;
        target
            .add_event_listener_with_callback(name, callback.as_ref().unchecked_ref())
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
        debug!("removing event listener");
        self.target
            .remove_event_listener_with_callback(self.name, self.callback.as_ref().unchecked_ref())
            .unwrap();
    }
}

pub trait EventTarget: Sized {
    fn handlers(&mut self) -> &mut Handlers;

    fn on<Ev, State, Updater>(mut self, key: Key<State>, updater: Updater) -> Self
    where
        Ev: 'static + Event,
        State: 'static,
        Updater: 'static + FnMut(&State, Ev) -> Option<State>,
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
    fn add_listener<Ev, State, Updater>(&mut self, key: Key<State>, mut updater: Updater)
    where
        Ev: Event,
        State: 'static,
        Updater: 'static + FnMut(&State, Ev) -> Option<State>,
    {
        self.inner.push(Box::new(move |target| -> EventHandle {
            EventHandle::new(target.to_owned(), key, updater)
        }));
    }

    pub(crate) fn apply(self, target: &web_sys::EventTarget) -> Vec<EventHandle> {
        self.inner
            .into_iter()
            .map(|mut handler| handler(target))
            .collect()
    }
}

impl Debug for Handlers {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Handlers").finish()
    }
}

pub struct ClickEvent(web_sys::MouseEvent);

impl AsRef<web_sys::Event> for ClickEvent {
    fn as_ref(&self) -> &web_sys::Event {
        self.0.as_ref()
    }
}

impl Event for ClickEvent {
    const NAME: &'static str = "click";
}
