use {
    crate::*,
    derive_builder::*,
    moxie::*,
    std::fmt::{Debug, Formatter, Result as FmtResult},
    stdweb::{traits::*, *},
    tracing::*,
};

pub fn on() -> HandlersBuilder {
    HandlersBuilder::default()
}

#[derive(Builder, Default)]
#[builder(pattern = "owned", setter(into, strip_option))]
pub struct Handlers {
    pub(crate) click: Option<Box<dyn FnMut() + 'static>>,
}

impl HandlersBuilder {
    pub fn set_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.click = Some(Some(Box::new(f)));
        self
    }
}

impl Debug for Handlers {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Handlers")
            .field("click", &self.click.as_ref().map(|_| "..."))
            .finish()
    }
}

impl PartialEq for Handlers {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
