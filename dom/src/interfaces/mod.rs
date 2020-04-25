//! Traits which correspond to the web platform's class interfaces.

macro_rules! attr_method {
    (
        $(#[$outer:meta])*
        $attr:ident
    ) => {
        $(#[$outer])*
        #[topo::nested]
        fn $attr(&self, to_set: impl Into<String>) -> &Self {
            self.attribute(stringify!($attr), to_set.into());
            self
        }
    };
}

macro_rules! bool_attr_method {
    (
        $(#[$outer:meta])*
        $attr:ident
    ) => {
        $(#[$outer])*
        #[topo::nested]
        fn $attr(&self, to_set: bool) -> &Self {
            if to_set {
                self.attribute(stringify!($attr), "".to_string());
            }
            self
        }
    };
}

pub mod element;
pub mod event_target;
pub mod global_events;
pub mod html_element;
pub mod node;
