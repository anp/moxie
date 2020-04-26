//! Element definitions generated from the listing on [MDN].
//!
//! [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element

use crate::{
    interfaces::node::{sealed::Memoized, Node},
    memo_node::MemoNode,
    prelude::*,
};
use augdom::event;

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
        impl HtmlElement for $ret {}
        impl Node for $ret {}

        impl Memoized for $ret {
            fn node(&self) -> &MemoNode {
                &self.inner
            }
        }

        // global event handlers
        impl GlobalEventHandler for $ret {}
        impl EventTarget<event::Abort> for $ret {}
        impl EventTarget<event::Blur> for $ret {}
        impl EventTarget<event::Cancel> for $ret {}
        impl EventTarget<event::Error> for $ret {}
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

/// A module for glob-importing all element creation functions, similar to the
/// global HTML namespace.
pub mod all {
    pub use super::{
        body,
        embedding::{embed, iframe, object, param, picture, source},
        forms::{
            button, datalist, fieldset, form, input, label, legend, meter, optgroup, option,
            output, progress, select, textarea,
        },
        html,
        interactive::{details, dialog, menu, summary},
        media::{area, audio, img, map, track, video},
        metadata::{base, head, link, meta, style, title},
        scripting::{canvas, noscript, script},
        sectioning::{
            address, article, aside, footer, h1, h2, h3, h4, h5, h6, header, hgroup, main, nav,
            section,
        },
        table::{caption, col, colgroup, table, tbody, td, tfoot, th, thead, tr},
        text_content::{blockquote, dd, div, dl, dt, figcaption, figure, hr, li, ol, p, pre, ul},
        text_semantics::{
            a, abbr, b, bdi, bdo, br, cite, code, data, del, dfn, em, i, ins, kbd, mark, q, rb, rp,
            rt, rtc, ruby, s, samp, small, span, strong, sub, sup, time, u, var, wbr,
        },
    };
}

pub mod embedding;
pub mod forms;
pub mod interactive;
pub mod media;
pub mod metadata;
pub mod scripting;
pub mod sectioning;
pub mod table;
pub mod text_content;
pub mod text_semantics;

element! {
    /// The [`<html>` element][mdn] represents the root (top-level element) of an HTML document,
    /// so it is also referred to as the *root element*. All other elements must be descendants of
    /// this element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/html
    html -> Root
}

element! {
    /// The [HTML `<body>` element][mdn] represents the content of an HTML document. There can be
    /// only one `<body>` element in a document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/body
    body -> Body
}

element! {
    /// The [HTML `<slot>` element][mdn]—part of the [Web Components][wc] technology suite—is a
    /// placeholder inside a web component that you can fill with your own markup, which lets you
    /// create separate DOM trees and present them together.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/slot
    /// [wc]: https://developer.mozilla.org/en-US/docs/Web/Web_Components
    slot -> Slot
}

element! {
    /// The [HTML Content Template (`<template>`) element][mdn] is a mechanism for holding [HTML]
    /// that is not to be rendered immediately when a page is loaded but may be instantiated
    /// subsequently during runtime using JavaScript.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/template
    /// [HTML]: https://developer.mozilla.org/en-US/docs/Glossary/HTML
    template -> Template
}
