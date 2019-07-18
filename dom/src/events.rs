use {
    crate::*,
    std::fmt::{Debug, Formatter, Result as FmtResult},
};

pub trait EventTarget: Sized {
    fn handlers(&mut self) -> &mut Handlers;

    fn on<Event, State, Updater>(mut self, key: Key<State>, updater: Updater) -> Self
    where
        Event: 'static + web::event::ConcreteEvent,
        State: 'static,
        Updater: 'static + FnMut(&State, Event) -> Option<State>,
    {
        self.handlers().add_listener(key, updater);
        self
    }
}

#[derive(Default)]
pub struct Handlers {
    inner: Vec<Box<dyn FnOnce(&web::EventTarget) + 'static>>,
}

impl Handlers {
    fn add_listener<Event, State, Updater>(&mut self, key: Key<State>, mut updater: Updater)
    where
        Event: web::event::ConcreteEvent,
        State: 'static,
        Updater: 'static + FnMut(&State, Event) -> Option<State>,
    {
        self.inner.push(Box::new(move |target| {
            target.add_event_listener(move |event: Event| {
                key.update(|prev| updater(prev, event));
            });
        }));
    }

    pub(crate) fn apply(self, target: &web::EventTarget) {
        for handler in self.inner {
            handler(target);
        }
    }
}

impl Debug for Handlers {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Handlers").finish()
    }
}
