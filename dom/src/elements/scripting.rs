//! Scripting

use crate::{
    interfaces::node::{sealed::Memoized, Node},
    memo_node::MemoNode,
    prelude::*,
};
use augdom::event;

html_element! {
    /// Use the [HTML `<canvas>` element][mdn] with either the [canvas scripting API][api] or the
    /// [WebGL API][gl] to draw graphics and animations.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas
    /// [api]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API
    /// [gl]: https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API
    canvas -> Canvas
}

html_element! {
    /// The [HTML `<noscript>` element][mdn] defines a section of HTML to be inserted if a script
    /// type on the page is unsupported or if scripting is currently turned off in the browser.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/noscript
    noscript -> NoScript
}

html_element! {
    /// The [HTML `<script>` element][mdn] is used to embed or reference executable code; this is
    /// typically used to embed or refer to JavaScript code.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script
    script -> Script
}

impl Script {
    attr_method! {
        /// Executes the script asynchronously.
        pub async_
    }

    attr_method! {
        /// Indicates that the script should be executed after the page has been parsed.
        pub defer
    }

    attr_method! {
        /// Defines the script language used in the element.
        pub language
    }
}
