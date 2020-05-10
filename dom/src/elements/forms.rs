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
    <button>
}

html_element! {
    /// The [HTML `<datalist>` element][mdn] contains a set of [`<option>`][option] elements that
    /// represent the values available for other controls.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/datalist
    /// [option]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/option
    <datalist>
}

html_element! {
    /// The [HTML `<fieldset>` element][mdn] is used to group several controls as well as labels
    /// ([`<label>`][label]) within a web form.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/fieldset
    /// [label]: <a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label" title="The HTML <label> element represents a caption for an item in a user interface.">
    <fieldset>
}

html_element! {
    /// The [HTML `<form>` element][mdn] represents a document section that contains interactive
    /// controls for submitting information to a web server.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form
    <form>
}

impl Form {
    attr_method! {
        /// The URI of a program that processes the information submitted via the form.
        pub action
    }

    attr_method! {
        /// Defines the content type of the form date when the method is POST.
        pub enctype
    }

    attr_method! {
        /// Defines which HTTP method to use when submitting the form. Can be GET (default) or POST.
        pub method
    }

    attr_method! {
        /// This attribute indicates that the form shouldn't be validated when submitted.
        pub novalidate
    }

    /// Space-separated character encodings the server accepts. The browser uses
    /// them in the order in which they are listed. The default value means
    /// the same encoding as the page.
    pub fn accept_charset(&self, to_set: String) -> &Self {
        self.attribute("accept-charset", to_set)
    }
}

html_element! {
    /// The [HTML `<input>` element][mdn] is used to create interactive controls for web-based forms
    /// in order to accept data from the user; a wide variety of types of input data and control
    /// widgets are available, depending on the device and [user agent].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input
    /// [user agent]: https://developer.mozilla.org/en-US/docs/Glossary/user_agent
    <input>
}

impl Input {
    attr_method! {
        /// The definition of 'media capture' in that specification.spec, specifies a new
        /// file can be captured.
        pub capture
    }

    attr_method! {
        /// Indicates whether the element should be checked on page load.
        pub checked
    }

    attr_method! {
        /// Identifies a list of pre-defined options to suggest to the user.
        pub list
    }

    attr_method! {
        /// Defines a regular expression which the element's value will be validated against.
        pub pattern
    }

    attr_method! {
        /// Valid for the numeric input types, including number, date/time input types, and range,
        /// the step attribute is a number that specifies the granularity that the value must adhere
        /// to.
        pub step
    }
}

html_element! {
    /// The [HTML `<label>` element][mdn] represents a caption for an item in a user interface.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label
    <label>
}

html_element! {
    /// The [HTML `<legend>` element][mdn] represents a caption for the content of its parent
    /// [`<fieldset>`][fieldset].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/legend
    /// [fieldset]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/fieldset
    <legend>
}

html_element! {
    /// The [HTML `<meter>` element][mdn] represents either a scalar value within a known range or a
    /// fractional value.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meter
    <meter>
}

impl Meter {
    attr_method! {
        /// Indicates the lower bound of the upper range.
        pub high(u32)
    }

    attr_method! {
        /// Indicates the upper bound of the lower range.
        pub low(u32)
    }

    attr_method! {
        /// Indicates the optimal numeric value.
        pub optimum(u32)
    }
}

html_element! {
    /// The [HTML `<optgroup>` element][mdn] creates a grouping of options within a
    /// [`<select>`][select] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/optgroup
    /// [select]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select
    <optgroup>
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
    <option>
}

impl super::forms::Option {
    attr_method! {
        /// Defines a value which will be selected on page load.
        pub selected
    }
}

html_element! {
    /// The [HTML Output element (`<output>`)][mdn] is a container element into which a site or app
    /// can inject the results of a calculation or the outcome of a user action.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/output
    <output>
}

html_element! {
    /// The [HTML `<progress>` element][progress] displays an indicator showing the completion
    /// progress of a task, typically displayed as a progress bar.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/progress
    <progress>
}

html_element! {
    /// The [HTML `<select>` element][mdn] represents a control that provides a menu of options.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select
    <select>
}

html_element! {
    /// The [HTML `<textarea>` element][mdn] represents a multi-line plain-text editing control,
    /// useful when you want to allow users to enter a sizeable amount of free-form text, for
    /// example a comment on a review or feedback form.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/textarea
    <textarea>
}

impl Textarea {
    attr_method! {
        /// Indicates whether the text should be wrapped.
        pub wrap
    }

    attr_method! {
        /// The visible width of the text control, in average character widths. If it is specified,
        /// it must be a positive integer. If it is not specified, the default value is 20.
        pub cols(u32)
    }

    attr_method! {
        /// Defines the number of rows in a text area.
        pub rows(u32)
    }
}
