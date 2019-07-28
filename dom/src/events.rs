use {
    crate::*,
    std::{
        fmt::{Debug, Formatter, Result as FmtResult},
        ops::Deref,
    },
    web_sys::Event as DomEvent,
};

pub trait Target<Ev, State, Updater>
where
    Ev: Event,
    Updater: 'static + FnMut(&State, Ev) -> Option<State>,
{
    type Output: Component;

    fn on(self, key: Key<State>, updater: Updater) -> Self::Output;
}

pub struct Handler<State, Updater> {
    key: Key<State>,
    updater: Updater,
}

impl<State, Updater> Debug for Handler<State, Updater>
where
    State: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        unimplemented!()
    }
}

pub trait Event: Deref<Target = DomEvent> {
    fn ty(&self) -> &'static str;
}

pub struct ClickEvent(web_sys::MouseEvent);

impl Deref for ClickEvent {
    type Target = DomEvent;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Event for ClickEvent {
    fn ty(&self) -> &'static str {
        "click"
    }
}
