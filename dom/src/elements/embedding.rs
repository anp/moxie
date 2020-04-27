//! In addition to regular multimedia content, HTML can include a variety of
//! other content, even if it's not always easy to interact with.

use crate::{
    interfaces::node::{sealed::Memoized, Node},
    memo_node::MemoNode,
    prelude::*,
};
use augdom::event;

html_element! {
    /// The [HTML `<embed>` element][mdn] embeds external content at the specified point in the
    /// document. This content is provided by an external application or other source of interactive
    /// content such as a browser plug-in.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/embed
    embed -> Embed
}

html_element! {
    /// The [HTML Inline Frame element (`<iframe>`)][mdn] represents a nested [browsing context],
    /// embedding another HTML page into the current one.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe
    /// [browsing context]: https://developer.mozilla.org/en-US/docs/Glossary/browsing_context
    iframe -> InlineFrame
}

impl InlineFrame {
    attr_method! {
        /// Specifies a feature-policy for the iframe.
        pub allow
    }

    attr_method! {
        /// Stops a document loaded in an iframe from using certain features (such as submitting forms
        /// or opening new windows).
        pub sandbox
    }

    attr_method! {
        /// Inline HTML to embed, overriding the `src` attribute. If a browser does not support the
        /// `srcdoc` attribute, it will fall back to the URL in the `src` attribute.
        pub srcdoc
    }
}

html_element! {
    /// The [HTML `<object>` element][mdn] represents an external resource, which can be treated as
    /// an image, a nested browsing context, or a resource to be handled by a plugin.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/object
    object -> Object
}

impl Object {
    attr_method! {
        /// Specifies the URL of the resource.
        pub data
    }
}

html_element! {
    /// The [HTML `<param>` element][param] defines parameters for an [`<object>`][object] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/param
    /// [object]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/object
    param -> Param
}

html_element! {
    /// The [HTML `<picture>` element][mdn] contains zero or more [`<source>`][source] elements and
    /// one [`<img>`][img] element to provide versions of an image for different display/device
    /// scenarios.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/picture
    /// [source]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source
    /// [img]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img
    picture -> Picture
}

html_element! {
    /// The [HTML `<source>` element][source] specifies multiple media resources for the
    /// [`<picture>`][picture], the [`<audio>`][audio] element, or the [`<video>`][video] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source
    /// [picture]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/picture
    /// [audio]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio
    /// [video]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video
    source -> Source
}
