//! Metadata contains information about the page. This includes information
//! about styles, scripts and data to help software (search engines, browsers,
//! etc.) use and render the page. Metadata for styles and scripts may be
//! defined in the page or link to another file that has the information.

use crate::{
    interfaces::node::{sealed::Memoized, Node},
    memo_node::MemoNode,
    prelude::*,
};
use augdom::event;

html_element! {
    /// The [HTML `<base> element`][mdn] specifies the base URL to use for all relative URLs
    /// contained within a document. There can be only one `<base>` element in a document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base
    <base>
}

html_element! {
    /// The [HTML `<head>` element][mdn] contains machine-readable information ([metadata]) about
    /// the document, like its [title], [scripts], and [style sheets].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/head
    /// [metadata]: https://developer.mozilla.org/en-US/docs/Glossary/metadata
    /// [title]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/title
    /// [scripts]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script
    /// [style sheets]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style
    <head>
}

html_element! {
    /// The [HTML External Resource Link element (`<link>`)][mdn] specifies relationships between
    /// the current document and an external resource. This element is most commonly used to link to
    /// [stylesheets], but is also used to establish site icons (both "favicon" style icons and
    /// icons for the home screen and apps on mobile devices) among other things.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/link
    /// [stylesheets]: https://developer.mozilla.org/en-US/docs/Glossary/CSS
    <link>
}

html_element! {
    /// The [HTML `<meta>` element][mdn] represents [metadata] that cannot be represented by other
    /// HTML meta-related elements, like [`<base>`], [`<link>`], [`<script>`], [`<style>`] or
    /// [`<title>`].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta
    /// [metadata]: https://developer.mozilla.org/en-US/docs/Glossary/Metadata
    /// [base]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base
    /// [link]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/link
    /// [script]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script
    /// [style]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style
    /// [title]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/title
    <meta>
}

impl Meta {
    attr_method! {
        /// A value associated with http-equiv or name depending on the context.
        pub content
    }

    attr_method! {
        /// Defines a pragma directive.
        pub http_equiv
    }
}

html_element! {
    /// The [HTML `<style>` element][mdn] contains style information for a document, or part of a
    /// document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style
    <style>
}

html_element! {
    /// The [HTML Title element (`<title>`)][mdn] defines the document's title that is shown in a
    /// [browser]'s title bar or a page's tab.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/title
    /// [browser]: https://developer.mozilla.org/en-US/docs/Glossary/Browser
    <title>
}
