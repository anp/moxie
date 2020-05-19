/// Compute the name of the HTML attribute from the name of the builder method.
macro_rules! attr_name {
    (accept_charset) => {
        "accept-charset"
    };
    (as_) => {
        "as"
    };
    (async_) => {
        "async"
    };
    (for_) => {
        "for"
    };
    (http_equiv) => {
        "http-equiv"
    };
    (current_time) => {
        "currentTime"
    };
    (loop_) => {
        "loop"
    };
    (type_) => {
        "type"
    };
    ($attr:ident) => {
        stringify!($attr)
    };
}

/// Stamps a *string* attribute method with the provided identifier as the name,
/// optionally passing docs.
macro_rules! attr_method {
    (
        $(#[$outer:meta])*
        $publicity:vis $attr:ident(bool)
    ) => {
        $(#[$outer])*
        #[topo::nested]
        #[must_use = "needs to be built"]
        $publicity fn $attr(self, to_set: bool) -> Self {
            #[allow(unused)]
            use crate::interfaces::element::Element;
            if to_set {
                self.attribute(attr_name!($attr), "")
            } else {
                self
            }
        }
    };
    (
        $(#[$outer:meta])*
        $publicity:vis $attr:ident
    ) => {
        attr_method! {
            $(#[$outer])*
            $publicity $attr(impl ToString)
        }
    };
    (
        $(#[$outer:meta])*
        $publicity:vis $attr:ident($arg:ty)
    ) => {
        $(#[$outer])*
        #[topo::nested]
        #[must_use = "needs to be built"]
        $publicity fn $attr(self, to_set: $arg) -> Self {
            #[allow(unused)]
            use crate::interfaces::element::Element;
            self.attribute(attr_name!($attr), to_set.to_string())
        }
    };
}

/// Define an element type.
macro_rules! element {
    (
        $(#[$outer:meta])*
        <$name:ident>
        $(attributes {$(
            $(#[$attr_meta:meta])*
            $attr:ident $(( $attr_ty:ty ))?
        )*})?
    ) => { paste::item! {
        $(#[$outer])*
        #[topo::nested]
        #[illicit::from_env(parent: &crate::memo_node::MemoNode)]
        pub fn $name() -> [< $name:camel >] {
            #[allow(unused)]
            use augdom::Dom;
            #[allow(unused)]
            use crate::interfaces::node::Node;

            let elem = moxie::prelude::memo(stringify!($name), |ty| {
                parent.raw_node().create_element(ty)
            });
            [< $name:camel >] { inner: crate::memo_node::MemoNode::new(elem) }
        }

        $(#[$outer])*
        #[must_use = "needs to be bound to a parent"]
        pub struct [< $name:camel >] {
            inner: crate::memo_node::MemoNode
        }

        impl crate::interfaces::element::Element for [< $name:camel >] {}
        impl crate::interfaces::node::Node for [< $name:camel >] {}

        impl crate::interfaces::node::sealed::Memoized for [< $name:camel >] {
            fn node(&self) -> &crate::memo_node::MemoNode {
                &self.inner
            }
        }

        $(impl [< $name:camel >] {
            $(attr_method! {
                $(#[$attr_meta])*
                pub $attr $(($attr_ty))?
            })*
        })?
    }};
}

/// Define an HTML element type, which is essentially an `element!` with the
/// `HtmlElement` and `GlobalEventHandler` traits.
macro_rules! html_element {
    (
        $(#[$outer:meta])*
        <$name:ident>
        $($rem:tt)*
    ) => { paste::item! {
        element! {
            $(#[$outer])*
            <$name>
            $($rem)*
        }

        impl crate::interfaces::html_element::HtmlElement for [< $name:camel >] {}
        impl crate::interfaces::global_events::GlobalEventHandler for [< $name:camel >] {}

        impl<E> crate::interfaces::event_target::EventTarget<E> for [< $name:camel >]
        where E: crate::interfaces::global_events::GlobalEvent {}
    }};
}

macro_rules! content_category {
    (
        $(#[$outer:meta])*
        $to_impl:ident: $(< $receives:ty >),+
    ) => {
        paste::item! {
            $(#[$outer])*
            pub trait $to_impl: crate::interfaces::node::Node {}

            $(impl $to_impl for [<$receives:camel>] {})+
        }
    };
}
