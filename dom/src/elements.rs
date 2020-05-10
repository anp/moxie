//! Element definitions generated from the listing on [MDN].
//!
//! [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element

use crate::{
    interfaces::node::{sealed::Memoized, Node},
    memo_node::MemoNode,
    prelude::*,
};
use augdom::event;

/// A module for glob-importing all element creation functions, similar to the
/// global HTML namespace.
pub mod html {
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

html_element! {
    /// The [`<html>` element][mdn] represents the root (top-level element) of an HTML document,
    /// so it is also referred to as the *root element*. All other elements must be descendants of
    /// this element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/html
    <html>
}

html_element! {
    /// The [HTML `<body>` element][mdn] represents the content of an HTML document. There can be
    /// only one `<body>` element in a document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/body
    <body>
}

html_element! {
    /// The [HTML `<slot>` element][mdn]—part of the [Web Components][wc] technology suite—is a
    /// placeholder inside a web component that you can fill with your own markup, which lets you
    /// create separate DOM trees and present them together.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/slot
    /// [wc]: https://developer.mozilla.org/en-US/docs/Web/Web_Components
    <slot>
}

html_element! {
    /// The [HTML Content Template (`<template>`) element][mdn] is a mechanism for holding [HTML]
    /// that is not to be rendered immediately when a page is loaded but may be instantiated
    /// subsequently during runtime using JavaScript.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/template
    /// [HTML]: https://developer.mozilla.org/en-US/docs/Glossary/HTML
    <template>
}
