//! In addition to regular multimedia content, HTML can include a variety of
//! other content, even if it's not always easy to interact with.

use crate::interfaces::security::ReferrerPolicy;

html_element! {
    /// The [HTML `<embed>` element][mdn] embeds external content at the specified point in the
    /// document. This content is provided by an external application or other source of interactive
    /// content such as a browser plug-in.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/embed
    <embed>

    attributes {
        /// The displayed height of the resource, in [CSS pixels]. This must be an absolute value;
        /// percentages are not allowed.
        ///
        /// [CSS pixels]: https://drafts.csswg.org/css-values/#px
        height

        /// The URL of the resource being embedded.
        src

        /// The [MIME type] to use to select the plug-in to instantiate.
        ///
        /// [MIME type]: https://developer.mozilla.org/en-US/docs/Glossary/MIME_type
        type_

        /// The displayed width of the resource, in [CSS pixels]. This must be an absolute value;
        /// percentages are not allowed.
        ///
        /// [CSS pixels]: https://drafts.csswg.org/css-values/#px
        width
    }
}

html_element! {
    /// The [HTML Inline Frame element (`<iframe>`)][mdn] represents a nested [browsing context],
    /// embedding another HTML page into the current one.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe
    /// [browsing context]: https://developer.mozilla.org/en-US/docs/Glossary/browsing_context
    <iframe>

    attributes {
        /// Specifies a feature policy for the <iframe>.
        allow

        /// The height of the frame in CSS pixels. Default is 150.
        height

        /// A targetable name for the embedded browsing context. This can be used in the target
        /// attribute of the <a>, <form>, or <base> elements; the formtarget attribute of the
        /// <input> or <button> elements; or the windowName parameter in the window.open() method.
        name

        /// Indicates which referrer to send when fetching the frame's resource.
        referrerpolicy(ReferrerPolicy)

        /// Applies extra restrictions to the content in the frame. The value of the attribute can
        /// either be empty to apply all restrictions, or space-separated tokens to lift particular
        /// restrictions:
        ///
        /// * allow-downloads-without-user-activation: Allows for downloads to occur without a
        ///   gesture from the user.
        /// * allow-forms: Allows the resource to submit forms. If this keyword is not used, form
        ///   submission is blocked.
        /// * allow-modals: Lets the resource open modal windows.
        /// * allow-orientation-lock: Lets the resource lock the screen orientation.
        /// * allow-pointer-lock: Lets the resource use the Pointer Lock API.
        /// * allow-popups: Allows popups (such as window.open(), target="_blank", or
        ///   showModalDialog()). If this keyword is not used, the popup will silently fail to open.
        /// * allow-popups-to-escape-sandbox: Lets the sandboxed document open new windows without
        ///   those windows inheriting the sandboxing. For example, this can safely sandbox an
        ///   advertisement without forcing the same restrictions upon the page the ad links to.
        /// * allow-presentation: Lets the resource start a presentation session.
        /// * allow-same-origin: If this token is not used, the resource is treated as being from a
        ///   special origin that always fails the same-origin policy.
        /// * allow-scripts: Lets the resource run scripts (but not create popup windows).
        /// * allow-storage-access-by-user-activation : Lets the resource request access to the
        ///   parent's storage capabilities with the Storage Access API.
        /// * allow-top-navigation: Lets the resource navigate the top-level browsing context (the
        ///   one named _top).
        /// * allow-top-navigation-by-user-activation: Lets the resource navigate the top-level
        ///   browsing context, but only if initiated by a user gesture.
        ///
        /// Notes about sandboxing:
        ///
        /// When the embedded document has the same origin as the embedding page, it is strongly
        /// discouraged to use both allow-scripts and allow-same-origin, as that lets the embedded
        /// document remove the sandbox attribute — making it no more secure than not using the
        /// sandbox attribute at all.
        ///
        /// Sandboxing is useless if the attacker can display content outside a sandboxed iframe —
        /// such as if the viewer opens the frame in a new tab. Such content should be also served
        /// from a separate origin to limit potential damage.
        sandbox

        /// The URL of the page to embed. Use a value of about:blank to embed an empty page that
        /// conforms to the same-origin policy. Also note that programatically removing an
        /// <iframe>'s src attribute (e.g. via Element.removeAttribute()) causes about:blank to be
        /// loaded in the frame in Firefox (from version 65), Chromium-based browsers, and
        /// Safari/iOS.
        src

        /// Inline HTML to embed, overriding the src attribute. If a browser does not support the
        /// srcdoc attribute, it will fall back to the URL in the src attribute.
        srcdoc

        /// The width of the frame in CSS pixels. Default is 300.
        width
    }
}

html_element! {
    /// The [HTML `<object>` element][mdn] represents an external resource, which can be treated as
    /// an image, a nested browsing context, or a resource to be handled by a plugin.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/object
    <object>

    attributes {
        /// Specifies the URL of the resource.
        data

        /// The form element, if any, that the object element is associated with (its form owner).
        /// The value of the attribute must be an ID of a <form> element in the same document.
        form

        /// The height of the displayed resource, in CSS pixels. No percentages.
        height

        /// The name of valid browsing context.
        name

        /// The content type of the resource specified by data. At least one of data and type must
        /// be defined.
        type_

        /// Indicates if the type attribute and the actual content type of the resource must match
        /// to be used.
        typemustmatch(bool)

        /// A hash-name reference to a <map> element; that is a '#' followed by the value of a name
        /// of a map element.
        usemap

        /// The width of the display resource, in CSS pixels. No percentages.
        width
    }
}

html_element! {
    /// The [HTML `<param>` element][param] defines parameters for an [`<object>`][object] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/param
    /// [object]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/object
    <param>

    attributes {
        /// Name of the parameter.
        name

        /// Specifies the value of the parameter.
        value
    }
}

html_element! {
    /// The [HTML `<picture>` element][mdn] contains zero or more [`<source>`][source] elements and
    /// one [`<img>`][img] element to provide versions of an image for different display/device
    /// scenarios.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/picture
    /// [source]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source
    /// [img]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img
    <picture>
}

html_element! {
    /// The [HTML `<source>` element][source] specifies multiple media resources for the
    /// [`<picture>`][picture], the [`<audio>`][audio] element, or the [`<video>`][video] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source
    /// [picture]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/picture
    /// [audio]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio
    /// [video]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video
    <source>

    attributes {
        /// Media query of the resource's intended media; this should be used only in a <picture>
        /// element.
        media

        /// Is a list of source sizes that describes the final rendered width of the image
        /// represented by the source. Each source size consists of a comma-separated list of media
        /// condition-length pairs. This information is used by the browser to determine, before
        /// laying the page out, which image defined in srcset to use. Please note that sizes will
        /// have its effect only if width dimension descriptors are provided with srcset instead of
        /// pixel ratio values (200w instead of 2x for example).
        ///
        /// The sizes attribute has an effect only when the <source> element is the direct child of
        /// a <picture> element.
        sizes

        /// Required for <audio> and <video>, address of the media resource. The value of this
        /// attribute is ignored when the <source> element is placed inside a <picture> element.
        src

        /// A list of one or more strings separated by commas indicating a set of possible images
        /// represented by the source for the browser to use. Each string is composed of:
        ///
        /// 1. One URL specifying an image.
        /// 2. A width descriptor, which consists of a string containing a positive integer directly
        ///    followed by "w", such as 300w. The default value, if missing, is the infinity.
        /// 3. A pixel density descriptor, that is a positive floating number directly followed by
        ///    "x". The default value, if missing, is 1x.
        ///
        /// Each string in the list must have at least a width descriptor or a pixel density
        /// descriptor to be valid. Among the list, there must be only one string containing the
        /// same tuple of width descriptor and pixel density descriptor. The browser chooses the
        /// most adequate image to display at a given point of time.
        ///
        /// The srcset attribute has an effect only when the <source> element is the direct child of
        /// a <picture> element.
        srcset

        /// The MIME media type of the resource, optionally with a codecs parameter.
        type_
    }
}
