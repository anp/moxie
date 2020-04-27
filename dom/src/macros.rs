/// Stamps a *string* attribute method with the provided identifier as the name,
/// optionally passing docs.
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

/// Stamps a *boolean* attribute method with the provided identifier as the
/// name, optionally passing docs.
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

/// Stamps an *unsigned integer* attribute method with the provided identifier
/// as the name, optionally passing docs.
macro_rules! unum_attr_method {
    (
        $(#[$outer:meta])*
        $attr:ident
    ) => {
        $(#[$outer])*
        #[topo::nested]
        fn $attr(&self, to_set: u32) -> &Self {
            self.attribute(stringify!($attr), to_set.to_string());
            self
        }
    };
}

/// Define an element type.
macro_rules! element {
    (
        $(#[$outer:meta])*
        $name:ident -> $ret:ident
    ) => {
        $(#[$outer])*
        #[topo::nested]
        #[illicit::from_env(parent: &MemoNode)]
        pub fn $name() -> $ret {
            let elem = memo(stringify!($name), |ty| {
                parent.raw_node().create_element(ty)
            });
            parent.ensure_child_attached(&elem);
            $ret { inner: MemoNode::new(elem) }
        }

        $(#[$outer])*
        pub struct $ret {
            inner: MemoNode
        }

        impl Element for $ret {}
        impl Node for $ret {}

        impl Memoized for $ret {
            fn node(&self) -> &MemoNode {
                &self.inner
            }
        }
    };
}

macro_rules! mass_bare_impl {
    ($to_impl:ty: $($receives:ty,)+) => {
        $(impl $to_impl for $receives {})+
    };
}

/// Define an HTML element type, which is essentially an `element!` with the
/// `HtmlElement` and `GlobalEventHandler` traits.
macro_rules! html_element {
    (
        $(#[$outer:meta])*
        $name:ident -> $ret:ident
    ) => {
        element! {
            $(#[$outer])*
            $name -> $ret
        }

        impl HtmlElement for $ret {}
        impl GlobalEventHandler for $ret {}
        impl EventTarget<event::Abort> for $ret {}
        impl EventTarget<event::Blur> for $ret {}
        impl EventTarget<event::Cancel> for $ret {}
        impl EventTarget<event::ErrorEvent> for $ret {}
        impl EventTarget<event::Focus> for $ret {}
        impl EventTarget<event::CanPlay> for $ret {}
        impl EventTarget<event::CanPlayThrough> for $ret {}
        impl EventTarget<event::Change> for $ret {}
        impl EventTarget<event::Click> for $ret {}
        impl EventTarget<event::CloseWebsocket> for $ret {}
        impl EventTarget<event::ContextMenu> for $ret {}
        impl EventTarget<event::CueChange> for $ret {}
        impl EventTarget<event::DoubleClick> for $ret {}
        impl EventTarget<event::Drag> for $ret {}
        impl EventTarget<event::DragEnd> for $ret {}
        impl EventTarget<event::DragEnter> for $ret {}
        impl EventTarget<event::DragExit> for $ret {}
        impl EventTarget<event::DragLeave> for $ret {}
        impl EventTarget<event::DragOver> for $ret {}
        impl EventTarget<event::DragStart> for $ret {}
        impl EventTarget<event::Dropped> for $ret {}
        impl EventTarget<event::DurationChange> for $ret {}
        impl EventTarget<event::Emptied> for $ret {}
        impl EventTarget<event::PlaybackEnded> for $ret {}
        impl EventTarget<event::GotPointerCapture> for $ret {}
        impl EventTarget<event::Input> for $ret {}
        impl EventTarget<event::Invalid> for $ret {}
        impl EventTarget<event::KeyDown> for $ret {}
        impl EventTarget<event::KeyPress> for $ret {}
        impl EventTarget<event::KeyUp> for $ret {}
        impl EventTarget<event::ResourceLoad> for $ret {}
        impl EventTarget<event::DataLoaded> for $ret {}
        impl EventTarget<event::MetadataLoaded> for $ret {}
        impl EventTarget<event::LoadEnd> for $ret {}
        impl EventTarget<event::LoadStart> for $ret {}
        impl EventTarget<event::LostPointerCapture> for $ret {}
        impl EventTarget<event::MouseEnter> for $ret {}
        impl EventTarget<event::MouseLeave> for $ret {}
        impl EventTarget<event::MouseMove> for $ret {}
        impl EventTarget<event::MouseOut> for $ret {}
        impl EventTarget<event::MouseOver> for $ret {}
        impl EventTarget<event::MouseUp> for $ret {}
        impl EventTarget<event::Wheel> for $ret {}
        impl EventTarget<event::Pause> for $ret {}
        impl EventTarget<event::Play> for $ret {}
        impl EventTarget<event::Playing> for $ret {}
        impl EventTarget<event::PointerDown> for $ret {}
        impl EventTarget<event::PointerMove> for $ret {}
        impl EventTarget<event::PointerUp> for $ret {}
        impl EventTarget<event::PointerCancel> for $ret {}
        impl EventTarget<event::PointerOver> for $ret {}
        impl EventTarget<event::PointerOut> for $ret {}
        impl EventTarget<event::PointerEnter> for $ret {}
        impl EventTarget<event::PointerLeave> for $ret {}
        impl EventTarget<event::Progress> for $ret {}
        impl EventTarget<event::PlaybackRateChange> for $ret {}
        impl EventTarget<event::FormReset> for $ret {}
        impl EventTarget<event::ViewResize> for $ret {}
        impl EventTarget<event::Scroll> for $ret {}
        impl EventTarget<event::Seeked> for $ret {}
        impl EventTarget<event::Seeking> for $ret {}
        impl EventTarget<event::Select> for $ret {}
        impl EventTarget<event::SelectionStart> for $ret {}
        impl EventTarget<event::SelectionChange> for $ret {}
        impl EventTarget<event::ContextMenuShow> for $ret {}
        impl EventTarget<event::Stalled> for $ret {}
        impl EventTarget<event::Submit> for $ret {}
        impl EventTarget<event::Suspend> for $ret {}
        impl EventTarget<event::TimeUpdate> for $ret {}
        impl EventTarget<event::VolumeChange> for $ret {}
        impl EventTarget<event::TransitionEnd> for $ret {}
        impl EventTarget<event::Waiting> for $ret {}
    };
}
