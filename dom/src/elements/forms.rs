//! HTML provides a number of elements which can be used together to create
//! forms which the user can fill out and submit to the Web site or application.
//! There's a great deal of further information about this available in the HTML
//! forms guide.

use crate::{
    interfaces::node::{sealed::Memoized, Node},
    memo_node::MemoNode,
    prelude::*,
};
use augdom::event;

html_element! {
    /// The [HTML `<button>` element][mdn] represents a clickable button, which can be used in
    /// [forms] or anywhere in a document that needs simple, standard button functionality.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button
    /// [forms]: https://developer.mozilla.org/en-US/docs/Learn/HTML/Forms
    button -> Button
}

html_element! {
    /// The [HTML `<datalist>` element][mdn] contains a set of [`<option>`][option] elements that
    /// represent the values available for other controls.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/datalist
    /// [option]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/option
    datalist -> DataList
}

html_element! {
    /// The [HTML `<fieldset>` element][mdn] is used to group several controls as well as labels
    /// ([`<label>`][label]) within a web form.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/fieldset
    /// [label]: <a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label" title="The HTML <label> element represents a caption for an item in a user interface.">
    fieldset -> FieldSet
}

html_element! {
    /// The [HTML `<form>` element][mdn] represents a document section that contains interactive
    /// controls for submitting information to a web server.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form
    form -> Form
}

html_element! {
    /// The [HTML `<input>` element][mdn] is used to create interactive controls for web-based forms
    /// in order to accept data from the user; a wide variety of types of input data and control
    /// widgets are available, depending on the device and [user agent].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input
    /// [user agent]: https://developer.mozilla.org/en-US/docs/Glossary/user_agent
    input -> Input
}

html_element! {
    /// The [HTML `<label>` element][mdn] represents a caption for an item in a user interface.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label
    label -> Label
}

html_element! {
    /// The [HTML `<legend>` element][mdn] represents a caption for the content of its parent
    /// [`<fieldset>`][fieldset].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/legend
    /// [fieldset]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/fieldset
    legend -> Legend
}

html_element! {
    /// The [HTML `<meter>` element][mdn] represents either a scalar value within a known range or a
    /// fractional value.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meter
    meter -> Meter
}

html_element! {
    /// The [HTML `<optgroup>` element][mdn] creates a grouping of options within a
    /// [`<select>`][select] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/optgroup
    /// [select]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select
    optgroup -> OptionGroup
}

html_element! {
    /// The [HTML `<option>` element][mdn] is used to define an item contained in a
    /// [`<select>`][select], an [`<optgroup>`][optgroup], or a [`<datalist>`][datalist] element. As
    /// such, `<option>` can represent menu items in popups and other lists of items in an HTML
    /// document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/option
    /// [select]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select
    /// [optgroup]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/optgroup
    /// [datalist]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/datalist
    option -> Option
}

html_element! {
    /// The [HTML Output element (`<output>`)][mdn] is a container element into which a site or app
    /// can inject the results of a calculation or the outcome of a user action.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/output
    output -> Output
}

html_element! {
    /// The [HTML `<progress>` element][progress] displays an indicator showing the completion
    /// progress of a task, typically displayed as a progress bar.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/progress
    progress -> Progress
}

html_element! {
    /// The [HTML `<select>` element][mdn] represents a control that provides a menu of options.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select
    select -> Select
}

html_element! {
    /// The [HTML `<textarea>` element][mdn] represents a multi-line plain-text editing control,
    /// useful when you want to allow users to enter a sizeable amount of free-form text, for
    /// example a comment on a review or feedback form.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/textarea
    textarea -> TextArea
}

impl TextArea {
    unum_attr_method! {
        /// The visible width of the text control, in average character widths. If it is specified,
        /// it must be a positive integer. If it is not specified, the default value is 20.
        cols
    }
}
