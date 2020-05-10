//! Traits which define attributes available to subsets of elements.

use crate::{
    elements::{
        embedding::*, forms::*, interactive::*, media::*, metadata::*, scripting::*, table::*,
        text_content::*, text_semantics::*,
    },
    prelude::*,
};

attr_trait! {
    /// List of types the server accepts, typically a file type.
    accept for
    <form>, <input>
}

attr_trait! {
    /// Specifies the horizontal alignment of the element.
    align for
    <caption>, <col>, <colgroup>, <hr>, <iframe>, <img>, <table>, <tbody>, <td>, <tfoot>, <th>,
    <thead>, <tr>
}

attr_trait! {
    /// Alternative text in case an image can't be displayed.
    alt for
    <area>, <img>, <input>
}

attr_trait! {
    /// Indicates whether controls in this form can by default have their values automatically
    /// completed by the browser.
    autocomplete for
    <form>, <input>, <select>, <textarea>
}

attr_trait! {
    /// The element should be automatically focused after the page loaded.
    autofocus for
    <button>, <input>, <select>, <textarea>
}

attr_trait! {
    /// The audio or video should play as soon as possible.
    autoplay for
    <audio>, <video>
}

attr_trait! {
    /// Contains the time range of already buffered media.
    buffered for
    <audio>, <video>
}

attr_trait! {
    /// Contains a URI which points to the source of the quote or change.
    cite for
    <blockquote>, <del>, <ins>, <q>
}

attr_trait! {
    /// The colspan attribute defines the number of columns a cell should span.
    colspan for
    <td>, <th>
}

attr_trait! {
    /// Indicates whether the browser should show playback controls to the user.
    controls for
    <audio>, <video>
}

attr_trait! {
    /// How the element handles cross-origin requests.
    crossorigin for
    <audio>, <img>, <link>, <script>, <video>
}

attr_trait! {
    /// Indicates the date and time associated with the element.
    datetime for
    <del>, <ins>, <time>
}

attr_trait! {
    /// The directionality of the element.
    dirname for
    <input>, <textarea>
}

attr_trait! {
    /// Indicates whether the user can interact with the element.
    disabled for
    <button>, <fieldset>, <input>, <optgroup>, <option>, <select>, <textarea>
}

attr_trait! {
    /// Indicates that the hyperlink is to be used for downloading a resource.
    download for
    <a>, <area>
}

attr_trait! {
    /// Describes elements which belongs to this one.
    for_ for
    <label>, <output>
}

attr_trait! {
    /// Indicates the form that is the owner of the element.
    form for
    <button>, <fieldset>, <input>, <label>, <meter>, <object>, <output>, <progress>, <select>,
    <textarea>
}

attr_trait! {
    /// Indicates the action of the element, overriding the action defined in the Form.
    formaction for
    <input>, <button>
}

attr_trait! {
    /// If the button/input is a submit button (type="submit"), this attribute sets the encoding
    /// type to use during form submission. If this attribute is specified, it overrides the enctype
    /// attribute of the button's form owner.
    formenctype for
    <button>, <input>
}

attr_trait! {
    /// If the button/input is a submit button (type="submit"), this attribute sets the submission
    /// method to use during form submission (GET, POST, etc.). If this attribute is specified, it
    /// overrides the method attribute of the button's form owner.
    formmethod for
    <button>, <input>
}

attr_trait! {
    /// If the button/input is a submit button (type="submit"), this boolean attribute specifies
    /// that the form is not to be validated when it is submitted. If this attribute is specified,
    /// it overrides the novalidate attribute of the button's form owner.
    formnovalidate for
    <button>, <input>
}

attr_trait! {
    /// If the button/input is a submit button (type="submit"), this attribute specifies the
    /// browsing context (for example, tab, window, or inline frame) in which to display the
    /// response that is received after submitting the form. If this attribute is specified, it
    /// overrides the target attribute of the button's form owner.
    formtarget for
    <button>, <input>
}

attr_trait! {
    /// IDs of the TableHeaderCell elements which applies to this element.
    headers for
    <td>, <th>
}

attr_trait! {
    /// The element's height attribute.
    height for
    <canvas>, <embed>, <iframe>, <img>, <input>, <object>, <video>
}

attr_trait! {
    /// The URL of a linked resource.
    href for
    <a>, <area>, <base>, <link>
}

attr_trait! {
    /// Specifies the language of the linked resource.
    hreflang for
    <a>, <area>, <link>
}

attr_trait! {
    /// Indicates the relative fetch priority for the resource.
    importance for
    <iframe>, <img>, <link>, <script>
}

attr_trait! {
    /// Specifies a Subresource Integrity value that allows browsers to verify what they fetch.
    integrity for
    <link>, <script>
}

attr_trait! {
    /// Specifies a user-readable title of the element.
    label for
    <optgroup>, <option>, <track>
}

attr_trait! {
    /// Indicates whether the media should start playing from the start when it's finished.
    loop_ for
    <audio>, <video>
}

attr_trait! {
    /// Indicates the maximum value allowed.
    max for
    <input>, <meter>, <progress>
}

attr_trait! {
    /// Defines the maximum number of characters allowed in the element.
    maxlength for
    <input>, <textarea>
}

attr_trait! {
    /// Defines the minimum number of characters allowed in the element.
    minlength for
    <input>, <textarea>
}

attr_trait! {
    /// Specifies a hint of the media for which the linked resource was designed.
    media for
    <a>, <area>, <link>, <source>, <style>
}

attr_trait! {
    /// Indicates the minimum value allowed.
    min for
    <input>, <meter>
}

attr_trait! {
    /// Indicates whether multiple values can be entered in an input of the type email or file.
    multiple for
    <input>, <select>
}

attr_trait! {
    /// Indicates whether the audio will be initially silenced on page load.
    muted for
    <audio>, <video>
}

attr_trait! {
    /// Name of the element. For example used by the server to identify the fields in form submits.
    name for
    <button>, <form>, <fieldset>, <iframe>, <input>, <object>, <output>, <select>,
    <textarea>, <map>, <meta>, <param>
}

attr_trait! {
    /// The ping attribute specifies a space-separated list of URLs to be notified if a user follows
    /// the hyperlink.
    ping for
    <a>, <area>
}

attr_trait! {
    /// Provides a hint to the user of what can be entered in the field.
    placeholder for
    <input>, <textarea>
}

attr_trait! {
    /// Indicates whether the whole resource, parts of it or nothing should be preloaded.
    preload for
    <audio>, <video>
}

attr_trait! {
    /// Indicates whether the element can be edited.
    readonly for
    <input>, <textarea>
}

attr_trait! {
    /// Specifies which referrer is sent when fetching the resource.
    referrerpolicy for
    <a>, <area>, <iframe>, <img>, <link>, <script>
}

attr_trait! {
    /// Specifies the relationship of the target object to the link object.
    rel for
    <a>, <area>, <link>
}

attr_trait! {
    /// Indicates whether this element is required to fill out or not.
    required for
    <input>, <select>, <textarea>
}

attr_trait! {
    /// Defines the number of rows a table cell should span over.
    rowspan for
    <td>, <th>
}

attr_trait! {
    /// Defines the width of the element (in pixels). If the element's type attribute is text or
    /// password then it's the number of characters.
    size for
    <input>, <select>
}

attr_trait! {
    /// One or more strings separated by commas, indicating a set of source sizes.
    sizes for
    <link>, <img>, <source>
}

attr_trait! {
    /// This attribute contains a positive integer indicating the number of consecutive columns the
    /// `<col>` element spans. If not present, its default value is 1.
    span for
    <col>, <colgroup>
}

attr_trait! {
    /// The URL of the embeddable content.
    src for
    <audio>, <embed>, <iframe>, <img>, <input>, <script>, <source>, <track>, <video>
}

attr_trait! {
    /// One or more responsive image candidates.
    srcset for
    <img>, <source>
}

attr_trait! {
    /// Specifies where to display a resource.
    target for
    <a>, <area>, <base>, <form>
}

attr_trait! {
    /// Defines the type of the element.
    type_ for
    <button>, <input>, <embed>, <object>, <script>, <source>, <style>, <menu>
}

attr_trait! {
    /// A hash-name reference to a `<map>` element; that is a '#' followed by the value of a name of
    /// a map element.
    usemap for
    <img>, <input>, <object>
}

attr_trait! {
    /// Defines a default value which will be displayed in the element on page load.
    value for
    <button>, <data>, <input>, <li>, <meter>, <option>, <progress>, <param>
}

attr_trait! {
    /// For the elements listed here, this establishes the element's width.
    width for
    <canvas>, <embed>, <iframe>, <img>, <input>, <object>, <video>
}
