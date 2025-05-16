//! Scripting

use crate::interfaces::security::ReferrerPolicy;

html_element! {
    /// Use the [HTML `<canvas>` element][mdn] with either the [canvas scripting API][api] or the
    /// [WebGL API][gl] to draw graphics and animations.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas
    /// [api]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API
    /// [gl]: https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API
    <canvas>

    categories {
        Flow, Phrasing, Embedded, Palpable
    }

    attributes {
        /// The height of the coordinate space in CSS pixels. Defaults to 150.
        height

        /// The width of the coordinate space in CSS pixels. Defaults to 300.
        width
    }
}

html_element! {
    /// The [HTML `<noscript>` element][mdn] defines a section of HTML to be inserted if a script
    /// type on the page is unsupported or if scripting is currently turned off in the browser.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/noscript
    <noscript>

    categories {
        Metadata, Flow, Phrasing
    }

    children {
        categories {
            Flow
        }
    }
}

html_element! {
    /// The [HTML `<script>` element][mdn] is used to embed or reference executable code; this is
    /// typically used to embed or refer to JavaScript code.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script
    <script>

    categories {
        Metadata, Flow, Phrasing
    }

    attributes {
        /// For classic scripts, if the async attribute is present, then the classic script will be
        /// fetched in parallel to parsing and evaluated as soon as it is available.
        ///
        /// For module scripts, if the async attribute is present then the scripts and all their
        /// dependencies will be executed in the defer queue, therefore they will get fetched in
        /// parallel to parsing and evaluated as soon as they are available.
        ///
        /// This attribute allows the elimination of parser-blocking JavaScript where the browser
        /// would have to load and evaluate scripts before continuing to parse. defer has a similar
        /// effect in this case.
        async_(bool)

        /// Normal script elements pass minimal information to the window.onerror for scripts which
        /// do not pass the standard CORS checks. To allow error logging for sites which use a
        /// separate domain for static media, use this attribute.
        crossorigin

        /// Indicates to a browser that the script is meant to be executed after the document has
        /// been parsed, but before firing DOMContentLoaded.
        ///
        /// Scripts with the defer attribute will prevent the DOMContentLoaded event from firing
        /// until the script has loaded and finished evaluating.
        ///
        /// This attribute must not be used if the src attribute is absent (i.e. for inline
        /// scripts), in this case it would have no effect.
        ///
        /// The defer attribute has no effect on module scripts — they defer by default.
        ///
        /// Scripts with the defer attribute will execute in the order in which they appear in the
        /// document.
        ///
        /// This attribute allows the elimination of parser-blocking JavaScript where the browser
        /// would have to load and evaluate scripts before continuing to parse. async has a similar
        /// effect in this case.
        defer(bool)

        /// This attribute contains inline metadata that a user agent can use to verify that a
        /// fetched resource has been delivered free of unexpected manipulation.
        integrity

        /// Indicates that the script should not be executed in browsers that support ES2015 modules
        /// — in effect, this can be used to serve fallback scripts to older browsers that do not
        /// support modular JavaScript code.
        nomodule(bool)

        /// A cryptographic nonce (number used once) to whitelist scripts in a script-src
        /// Content-Security-Policy. The server must generate a unique nonce value each time it
        /// transmits a policy. It is critical to provide a nonce that cannot be guessed as
        /// bypassing a resource's policy is otherwise trivial.
        nonce

        /// Indicates which referrer to send when fetching the script, or resources fetched by the
        /// script.
        referrerpolicy(ReferrerPolicy)

        /// This attribute specifies the URI of an external script; this can be used as an
        /// alternative to embedding a script directly within a document.
        src

        /// This attribute indicates the type of script represented. The value of this attribute
        /// will be in one of the following categories:
        ///
        /// * Omitted or a JavaScript MIME type: This indicates the script is JavaScript. The HTML5
        ///   specification urges authors to omit the attribute rather than provide a redundant MIME
        ///   type.
        /// * `module`: Causes the code to be treated as a JavaScript module. The processing of the
        ///   script contents is not affected by the charset and defer attributes. Unlike classic
        ///   scripts, module scripts require the use of the CORS protocol for cross-origin
        ///   fetching.
        /// * Any other value: The embedded content is treated as a data block which won't be
        ///   processed by the browser. Developers must use a valid MIME type that is not a
        ///   JavaScript MIME type to denote data blocks. The src attribute will be ignored.
        type_
    }
}

only_text_children! { <script> }
