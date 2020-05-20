//! Metadata contains information about the page. This includes information
//! about styles, scripts and data to help software (search engines, browsers,
//! etc.) use and render the page. Metadata for styles and scripts may be
//! defined in the page or link to another file that has the information.

html_element! {
    /// The [HTML `<base> element`][mdn] specifies the base URL to use for all relative URLs
    /// contained within a document. There can be only one `<base>` element in a document.
    ///
    /// If either of its inherent attributes are specified, this element must come before other
    /// elements with attributes whose values are URLs, such as <link>’s href attribute.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base
    <base>

    categories {
        Metadata
    }

    attributes {
        /// The base URL to be used throughout the document for relative URLs. Absolute and relative
        /// URLs are allowed.
        href

        /// A keyword or author-defined name of the default browsing context to display the result
        /// when links or forms cause navigation, for <a> or <form> elements without an explicit
        /// target attribute. The attribute value targets a browsing context (such as a tab, window,
        /// or <iframe>).
        ///
        /// The following keywords have special meanings:
        ///
        /// * `_self`: Load the result into the same browsing context as the current one. (This is
        ///   the default.)
        /// * `_blank`: Load the result into a new, unnamed browsing context.
        /// * `_parent`: Load the result into the parent browsing context of the current one. (If
        ///   the current page is inside a frame.) If there is no parent, behaves the same way as
        ///   _self.
        /// * `_top`: Load the result into the topmost browsing context (that is, the browsing
        ///   context that is an ancestor of the current one, and has no parent). If there is no
        ///   parent, behaves the same way as _self.
        target
    }
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

    children {
        categories {
            Metadata
        }
    }
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

    categories {
        Metadata,
        // if `itemprop` is present:
        Flow, Phrasing
    }

    attributes {
        /// This attribute is only used when rel="preload" or rel="prefetch" has been set on the
        /// <link> element. It specifies the type of content being loaded by the <link>, which is
        /// necessary for request matching, application of correct content security policy, and
        /// setting of correct Accept request header. Furthermore, rel="preload" uses this as a
        /// signal for request prioritization. The table below lists the valid values for this
        /// attribute and the elements or resources they apply to.
        ///
        /// | Value    | Applies To                                                              |
        /// | -------- | ----------------------------------------------------------------------- |
        /// | audio    | <audio> elements                                                        |
        /// | document | <iframe> and <frame> elements                                           |
        /// | embed    | <embed> elements                                                        |
        /// | fetch    | fetch, XHR (also requires <link> to contain the crossorigin attribute.) |
        /// | font     | CSS @font-face                                                          |
        /// | image    | <img> and <picture> elements with srcset or imageset attributes,        |
        /// |          | SVG <image> elements, CSS *-image rules                                 |
        /// | object   | <object> elements                                                       |
        /// | script   | <script> elements, Worker importScripts                                 |
        /// | style    | <link rel=stylesheet> elements, CSS @import                             |
        /// | track    | <track> elements                                                        |
        /// | video    | <video> elements                                                        |
        /// | worker   | Worker, SharedWorker                                                    |
        as_

        /// This enumerated attribute indicates whether CORS must be used when fetching the
        /// resource. CORS-enabled images can be reused in the <canvas> element without being
        /// tainted. The allowed values are:
        ///
        /// * `anonymous`: A cross-origin request (i.e. with an Origin HTTP header) is performed,
        ///   but no credential is sent (i.e. no cookie, X.509 certificate, or HTTP Basic
        ///   authentication). If the server does not give credentials to the origin site (by not
        ///   setting the Access-Control-Allow-Origin HTTP header) the resource will be tainted and
        ///   its usage restricted.
        /// * `use-credentials`: A cross-origin request (i.e. with an Origin HTTP header) is
        ///   performed along with a credential sent (i.e. a cookie, certificate, and/or HTTP Basic
        ///   authentication is performed). If the server does not give credentials to the origin
        ///   site (through Access-Control-Allow-Credentials HTTP header), the resource will be
        ///   tainted and its usage restricted.
        ///
        /// If the attribute is not present, the resource is fetched without a CORS request (i.e.
        /// without sending the Origin HTTP header), preventing its non-tainted usage. If invalid,
        /// it is handled as if the enumerated keyword anonymous was used.
        crossorigin

        /// For rel="stylesheet" only, the disabled Boolean attribute indicates whether or not the
        /// described stylesheet should be loaded and applied to the document. If disabled is
        /// specified in the HTML when it is loaded, the stylesheet will not be loaded during page
        /// load. Instead, the stylesheet will be loaded on-demand, if and when the disabled
        /// attribute is changed to false or removed.
        ///
        /// Once the stylesheet has been loaded, however, changes made to the value of the disabled
        /// property no longer have any relationship to the value of the StyleSheet.disabled
        /// property. Changing the value of this property instead simply enables and disables the
        /// stylesheet form being applied to the document.
        ///
        /// This differs from StyleSheet's disabled property; changing it to true removes the
        /// stylesheet from the document's document.styleSheets list, and doesn't automatically
        /// reload the stylesheet when it's toggled back to false.
        disabled

        /// This attribute specifies the URL of the linked resource. A URL can be absolute or
        /// relative.
        href

        /// This attribute indicates the language of the linked resource. It is purely advisory.
        /// Allowed values are determined by BCP47. Use this attribute only if the href attribute is
        /// present.
        hreflang

        /// This attribute specifies the media that the linked resource applies to. Its value must
        /// be a media type / media query. This attribute is mainly useful when linking to external
        /// stylesheets — it allows the user agent to pick the best adapted one for the device it
        /// runs on.
        media

        /// This attribute names a relationship of the linked document to the current document. The
        /// attribute must be a space-separated list of link type values.
        rel

        /// This attribute defines the sizes of the icons for visual media contained in the
        /// resource. It must be present only if the rel contains a value of icon or a non-standard
        /// type such as Apple's apple-touch-icon. It may have the following values:
        ///
        /// * `any`, meaning that the icon can be scaled to any size as it is in a vector format,
        ///   like image/svg+xml.
        /// * a white-space separated list of sizes, each in the format <width in pixels>x<height in
        ///   pixels> or <width in pixels>X<height in pixels>. Each of these sizes must be contained
        ///   in the resource.
        ///
        /// Note: Most icon formats are only able to store one single icon; therefore most of the
        /// time the sizes attribute contains only one entry. MS's ICO format does, as well as
        /// Apple's ICNS. ICO is more ubiquitous, so you should use this format if cross-browser
        /// support is a concern (especially for old IE versions).
        sizes

        /// The title attribute has special semantics on the <link> element. When used on a
        /// <link rel="stylesheet"> it defines a preferred or an alternate stylesheet. Incorrectly
        /// using it may cause the stylesheet to be ignored.
        title

        /// This attribute is used to define the type of the content linked to. The value of the
        /// attribute should be a MIME type such as text/html, text/css, and so on. The common use
        /// of this attribute is to define the type of stylesheet being referenced (such as
        /// text/css), but given that CSS is the only stylesheet language used on the web, not only
        /// is it possible to omit the type attribute, but is actually now recommended practice. It
        /// is also used on rel="preload" link types, to make sure the browser only downloads file
        /// types that it supports.
        type_
    }
}

html_element! {
    /// The [HTML `<meta>` element][mdn] represents [metadata] that cannot be represented by other
    /// HTML meta-related elements, like [`<base>`], [`<link>`], [`<script>`], [`<style>`] or
    /// [`<title>`].
    ///
    /// Note: the attribute `name` has a specific meaning for the <meta> element, and the `itemprop`
    /// attribute must not be set on the same <meta> element that has any existing name,
    /// `http-equiv` or `charset` attributes.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta
    /// [metadata]: https://developer.mozilla.org/en-US/docs/Glossary/Metadata
    /// [base]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base
    /// [link]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/link
    /// [script]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script
    /// [style]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style
    /// [title]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/title
    <meta>

    categories {
        Metadata,
        // if `itemprop` is present:
        Flow, Phrasing
    }

    attributes {
        /// This attribute declares the document's character encoding. If the attribute is present,
        /// its value must be an ASCII case-insensitive match for the string "utf-8".
        charset

        /// This attribute contains the value for the http-equiv or name attribute, depending on
        /// which is used.
        content

        /// Defines a pragma directive. The attribute is named http-equiv(alent) because all the
        /// allowed values are names of particular HTTP headers:
        ///
        /// * `content-security-policy`: Allows page authors to define a content policy for the
        ///   current page. Content policies mostly specify allowed server origins and script
        ///   endpoints which help guard against cross-site scripting attacks.
        /// * `content-type`: If specified, the content attribute must have the value
        ///   `text/html; charset=utf-8`. Note: Can only be used in documents served with a
        ///   text/html MIME type — not in documents served with an XML MIME type.
        /// * `default-style`: Sets the name of the default CSS style sheet set.
        /// * `x-ua-compatible`: If specified, the content attribute must have the value "IE=edge".
        ///   User agents are required to ignore this pragma.
        /// * `refresh`: This instruction specifies:
        ///   * The number of seconds until the page should be reloaded - only if the content
        ///     attribute contains a positive integer.
        ///   * The number of seconds until the page should redirect to another - only if the
        ///     content attribute contains a positive integer followed by the string ';url=', and a
        ///     valid URL.
        ///   * Accessibility concerns: Pages set with a refresh value run the risk of having the
        ///     time interval being too short. People navigating with the aid of assistive
        ///     technology such as a screen reader may be unable to read through and understand the
        ///     page's content before being automatically redirected. The abrupt, unannounced
        ///     updating of the page content may also be disorienting for people experiencing low
        ///     vision conditions.
        http_equiv

        /// The name and content attributes can be used together to provide document metadata in
        /// terms of name-value pairs, with the name attribute giving the metadata name, and the
        /// content attribute giving the value.
        ///
        /// See [standard metadata names] for details about the set of standard metadata names
        /// defined in the HTML specification.
        ///
        /// [standard metadata names]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta/name
        name
    }
}

html_element! {
    /// The [HTML `<style>` element][mdn] contains style information for a document, or part of a
    /// document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style
    <style>

    categories {
        Metadata,
        Flow // if the `scoped` attribute is present
    }

    attributes {
        /// This attribute defines which media the style should be applied to. Its value is a media
        /// query, which defaults to all if the attribute is missing.
        media

        /// A cryptographic nonce (number used once) used to whitelist inline styles in a style-src
        /// Content-Security-Policy. The server must generate a unique nonce value each time it
        /// transmits a policy. It is critical to provide a nonce that cannot be guessed as
        /// bypassing a resource’s policy is otherwise trivial.
        nonce

        /// This attribute specifies alternative style sheet sets.
        title
    }
}

only_text_children! { <style> }

html_element! {
    /// The [HTML Title element (`<title>`)][mdn] defines the document's title that is shown in a
    /// [browser]'s title bar or a page's tab.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/title
    /// [browser]: https://developer.mozilla.org/en-US/docs/Glossary/Browser
    <title>

    categories {
        Metadata
    }
}

only_text_children! { <title> }
