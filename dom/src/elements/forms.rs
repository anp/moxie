//! HTML provides a number of elements which can be used together to create
//! forms which the user can fill out and submit to the Web site or application.
//! There's a great deal of further information about this available in the HTML
//! forms guide.

html_element! {
    /// The [HTML `<button>` element][mdn] represents a clickable button, which can be used in
    /// [forms] or anywhere in a document that needs simple, standard button functionality.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button
    /// [forms]: https://developer.mozilla.org/en-US/docs/Learn/HTML/Forms
    <button>

    attributes {
        /// Specifies that the button should have input focus when the page loads. Only one element
        /// in a document can have this attribute.
        autofocus(bool)

        /// Prevents the user from interacting with the button: it cannot be pressed or focused.
        disabled(bool)

        /// The <form> element to associate the button with (its form owner). The value of this
        /// attribute must be the id of a <form> in the same document. (If this attribute is not
        /// set, the <button> is associated with its ancestor <form> element, if any.)
        ///
        /// This attribute lets you associate <button> elements to <form>s anywhere in the document,
        /// not just inside a <form>. It can also override an ancestor <form> element.
        form

        /// The URL that processes the information submitted by the button. Overrides the action
        /// attribute of the button's form owner. Does nothing if there is no form owner.
        formaction

        /// If the button is a submit button (it's inside/associated with a <form> and doesn't have
        /// type="button"), specifies how to encode the form data that is submitted. Possible
        /// values:
        ///
        /// * application/x-www-form-urlencoded: The default if the attribute is not used.
        /// * multipart/form-data: Use to submit <input> elements with their type attributes set to
        ///   file.
        /// * text/plain: Specified as a debugging aid; shouldn’t be used for real form submission.
        ///
        /// If this attribute is specified, it overrides the enctype attribute of the button's form
        /// owner.
        formenctype

        /// If the button is a submit button (it's inside/associated with a <form> and doesn't have
        /// type="button"), this attribute specifies the HTTP method used to submit the form.
        /// Possible values:
        ///
        /// * post: The data from the form are included in the body of the HTTP request when sent to
        ///   the server. Use when the form contains information that shouldn’t be public, like
        ///   login credentials.
        /// * get: The form data are appended to the form's action URL, with a ? as a separator, and
        ///   the resulting URL is sent to the server. Use this method when the form has no side
        ///   effects, like search forms.
        ///
        /// If specified, this attribute overrides the method attribute of the button's form owner.
        formmethod

        /// If the button is a submit button, specifies that the form is not to be validated when it
        /// is submitted. If this attribute is specified, it overrides the novalidate attribute of
        /// the button's form owner.
        ///
        /// This attribute is also available on <input type="image"> and <input type="submit">
        /// elements.
        formnovalidate(bool)

        /// If the button is a submit button, this attribute is a author-defined name or
        /// standardized, underscore-prefixed keyword indicating where to display the response from
        /// submitting the form. This is the name of, or keyword for, a browsing context (a tab,
        /// window, or <iframe>). If this attribute is specified, it overrides the target attribute
        /// of the button's form owner. The following keywords have special meanings:
        ///
        /// * _self: Load the response into the same browsing context as the current one.
        ///   This is the default if the attribute is not specified.
        /// * _blank: Load the response into a new unnamed browsing context — usually a new tab or
        ///   window, depending on the user’s browser settings.
        /// * _parent: Load the response into the parent browsing context of the current one. If
        ///   there is no parent, this option behaves the same way as _self.
        /// * _top: Load the response into the top-level browsing context (that is, the browsing
        ///   context that is an ancestor of the current one, and has no parent). If there is no
        ///   parent, this option behaves the same way as _self.
        formtarget

        /// The name of the button, submitted as a pair with the button’s value as part of the form
        /// data.
        name

        /// The default behavior of the button. Possible values are:
        ///
        /// * submit: The button submits the form data to the server. This is the default if the
        ///   attribute is not specified for buttons associated with a <form>, or if the attribute
        ///   is an empty or invalid value.
        /// * reset: The button resets all the controls to their initial values, like
        ///   <input type="reset">. (This behavior tends to annoy users.)
        /// * button: The button has no default behavior, and does nothing when pressed by default.
        ///   It can have client-side scripts listen to the element's events, which are triggered
        ///   when the events occur.
        type_

        /// Defines the value associated with the button’s name when it’s submitted with the form
        /// data. This value is passed to the server in params when the form is submitted.
        value
    }
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
    /// [label]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label
    <fieldset>

    attributes {
        /// If this Boolean attribute is set, all form controls that are descendants of the
        /// <fieldset> are disabled, meaning they are not editable and won't be submitted along with
        /// the <form>. They won't receive any browsing events, like mouse clicks or focus-related
        /// events. By default browsers display such controls grayed out. Note that form elements
        /// inside the <legend> element won't be disabled.
        disabled

        /// This attribute takes the value of the id attribute of a <form> element you want the
        /// <fieldset> to be part of, even if it is not inside the form.
        form

        /// The name associated with the group.
        ///
        /// Note: The caption for the fieldset is given by the first <legend> element inside it.
        name
    }
}

html_element! {
    /// The [HTML `<form>` element][mdn] represents a document section that contains interactive
    /// controls for submitting information to a web server.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form
    <form>
    attributes {
        /// Space-separated [character encodings] the server accepts. The browser uses
        /// them in the order in which they are listed. The default value means
        /// the same encoding as the page.
        ///
        /// [character encodings]: https://developer.mozilla.org/en-US/docs/Web/Guide/Localizations_and_character_encodings
        accept_charset

        /// The URI of a program that processes the information submitted via the form.
        action

        /// Indicates whether input elements can by default have their values automatically
        /// completed by the browser. autocomplete attributes on form elements override it on
        /// <form>. Possible values:
        ///
        /// * off: The browser may not automatically complete entries. (Browsers tend to ignore this
        ///   for suspected login forms; see The autocomplete attribute and login fields.)
        /// * on: The browser may automatically complete entries.
        autocomplete

        /// If the value of the method attribute is post, enctype is the MIME type of the form
        /// submission. Possible values:
        ///
        /// * application/x-www-form-urlencoded: The default value.
        /// * multipart/form-data: Use this if the form contains <input> elements with type=file.
        /// * text/plain: Introduced by HTML5 for debugging purposes.
        ///
        /// This value can be overridden by formenctype attributes on <button>,
        /// <input type="submit">, or <input type="image"> elements.
        enctype

        /// The HTTP method to submit the form with. Possible values:
        ///
        /// * post: The POST method; form data sent as the request body.
        /// * get: The GET method; form data appended to the action URL with a ? separator. Use this
        ///   method when the form has no side-effects.
        /// * dialog: When the form is inside a <dialog>, closes the dialog on submission.
        ///
        /// This value is overridden by formmethod attributes on <button>, <input type="submit">, or
        /// <input type="image"> elements.
        method

        /// Indicates that the form shouldn't be validated when submitted. If this attribute is not
        /// set (and therefore the form is validated), it can be overridden by a formnovalidate
        /// attribute on a <button>, <input type="submit">, or <input type="image"> element
        /// belonging to the form.
        novalidate(bool)

        /// Creates a hyperlink or annotation depending on the value.
        rel

        /// Indicates where to display the response after submitting the form. It is a name/keyword
        /// for a browsing context (for example, tab, window, or iframe). The following keywords
        /// have special meanings:
        ///
        /// * _self (default): Load into the same browsing context as the current one.
        /// * _blank: Load into a new unnamed browsing context.
        /// * _parent: Load into the parent browsing context of the current one. If no parent,
        ///   behaves the same as _self.
        /// * _top: Load into the top-level browsing context (i.e., the browsing context that is an
        ///   ancestor of the current one and has no parent). If no parent, behaves the same as
        ///   _self.
        ///
        /// This value can be overridden by a formtarget attribute on a <button>,
        /// <input type="submit">, or <input type="image"> element.
        target
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
    attributes {
        /// Valid for the file input type only, the accept property defines which file types are
        /// selectable in a file upload control. See the file input type.
        accept

        /// Valid for the image button only, the alt attribute provides alternative text for the
        /// image, displaying the value of the attribute if the image src is missing or otherwise
        /// fails to load. See the image input type.
        alt

        /// The autocomplete attribute takes as its value a space-separated string that describes
        /// what, if any, type of autocomplete functionality the input should provide. A typical
        /// implementation of autocomplete simply recalls previous values entered in the same input
        /// field, but more complex forms of autocomplete can exist. For instance, a browser could
        /// integrate with a device's contacts list to autocomplete email addresses in an email
        /// input field. See Values in The HTML autocomplete attribute for permitted values.
        ///
        /// The autocomplete attribute is valid on hidden, text, search, url, tel, email, date,
        /// month, week, time, datetime-local, number, range, color, and password. This attribute
        /// has no effect on input types that do not return numeric or text data, being valid for
        /// all input types except checkbox, radio, file, or any of the button types.
        ///
        /// See The HTML autocomplete attribute for additional information, including information on
        /// password security and how autocomplete is slightly different for hidden than for other
        /// input types.
        autocomplete

        /// Indicates if present that the input should automatically have focus when the page has
        /// finished loading (or when the <dialog> containing the element has been displayed).
        ///
        /// Note: An element with the autofocus attribute may gain focus before the DOMContentLoaded
        /// event is fired.
        ///
        /// No more than one element in the document may have the autofocus attribute. The autofocus
        /// attribute cannot be used on inputs of type hidden, since hidden inputs cannot be
        /// focused.
        ///
        /// If put on more than one element, the first one with the attribute receives focus.
        ///
        /// Warning: Automatically focusing a form control can confuse visually-impaired people
        /// using screen-reading technology and people with cognitive impairments. When autofocus is
        /// assigned, screen-readers "teleport" their user to the form control without warning them
        /// beforehand.
        ///
        /// For better usability, avoid using autofocus. Automatically focusing on a form control
        /// can cause the page to scroll on load. The focus can also cause dynamic keyboards to
        /// display on some touch devices. While a screen reader will announce the label of the form
        /// control receiving focus, the screen reader  will not announce anything before the label,
        /// and the sighted user on a small device will equally miss the context created by the
        /// preceding content.
        autofocus(bool)

        /// Introduced in the HTML Media Capture specification and valid for the file input type
        /// only, the capture attribute defines which media—microphone, video, or camera—should be
        /// used to capture a new file for upload with file upload control in supporting scenarios.
        /// See the file input type.
        capture

        /// Valid for both radio and checkbox types, checked is a Boolean attribute. If present on a
        /// radio type, it indicates that that radio button is the currently selected one in the
        /// group of same-named radio buttons. If present on a checkbox type, it indicates that the
        /// checkbox is checked by default (when the page loads). It does not indicate whether this
        /// checkbox is currently checked: if the checkbox’s state is changed, this content
        /// attribute does not reflect the change. (Only the HTMLInputElement’s checked IDL
        /// attribute is updated.)
        ///
        /// Note: Unlike other input controls, a checkboxes and radio buttons value are only
        /// included in the submitted data if they are currently checked. If they are, the name and
        /// the value(s) of the checked controls are submitted.
        ///
        /// For example, if a checkbox whose name is fruit has a value of cherry, and the checkbox
        /// is checked, the form data submitted will include fruit=cherry. If the checkbox isn't
        /// active, it isn't listed in the form data at all. The default value for checkboxes and
        /// radio buttons is on.
        checked

        /// Valid for text and search input types only, the dirname attribute enables the submission
        /// of the directionality of the element. When included, the form control will submit with
        /// two name/value pairs: the first being the name and value, the second being the value of
        /// the dirname as the name with the value of ltr or rtl being set by the browser.
        dirname

        /// If present indicates that the user should not be able to interact with the input.
        /// Disabled inputs are typically rendered with a dimmer color or using some other form of
        /// indication that the field is not available for use.
        ///
        /// Specifically, disabled inputs do not receive the click event, and disabled inputs are
        /// not submitted with the form.
        disabled(bool)

        /// A string specifying the <form> element with which the input is associated (that is, its
        /// form owner). This string's value, if present, must match the id of a <form> element in
        /// the same document. If this attribute isn't specified, the <input> element is associated
        /// with the nearest containing form, if any.
        ///
        /// The form attribute lets you place an input anywhere in the document but have it included
        /// with a form elsewhere in the document.
        ///
        /// Note: An input can only be associated with one form.
        form

        /// Valid for the image and submit input types only. See the submit input type for more
        /// information.
        formaction

        /// Valid for the image and submit input types only. See the submit input type for more
        /// information.
        formenctype

        /// Valid for the image and submit input types only. See the submit input type for more
        /// information.
        formmethod

        /// Valid for the image and submit input types only. See the submit input type for more
        /// information.
        formnovalidate

        /// Valid for the image and submit input types only. See the submit input type for more
        /// information.
        formtarget

        /// Valid for the image input button only, the height is the height of the image file to
        /// display to represent the graphical submit button. See the image input type.
        height

        /// Global attribute valid for all elements, including all the input types, it defines a
        /// unique identifier (ID) which must be unique in the whole document. Its purpose is to
        /// identify the element when linking. The value is used as the value of the <label>'s for
        /// attribute to link the label with the form control. See the the label element below.
        id

        /// Global value valid for all elements, it provides a hint to browsers as to the type of
        /// virtual keyboard configuration to use when editing this element or its contents. Values
        /// include none, text, tel, url, email, numeric, decimal, and search.
        inputmode

        /// The values of the list attribute is the id of a <datalist> element located in the same
        /// document. The <datalist>  provides a list of predefined values to suggest to the user
        /// for this input. Any values in the list that are not compatible with the type are not
        /// included in the suggested options.  The values provided are suggestions, not
        /// requirements: users can select from this predefined list or provide a different value.
        ///
        /// It is valid on text, search, url, tel, email, date, month, week, time, datetime-local,
        /// number, range, and color.
        ///
        /// Per the specifications, the list attribute is not supported by the hidden, password,
        /// checkbox, radio, file, or any of the button types.
        ///
        /// Depending on the browser, the user may see a custom color palette suggested, tic marks
        /// along a range, or even a input that opens like a select but allows for non-listed
        /// values. Check out the browser compatibility table for the other input types.
        ///
        /// See the <datalist> element.
        list

        /// Valid for date, month, week, time, datetime-local, number, and range, it defines the
        /// greatest value in the range of permitted values. If the value entered into the element
        /// exceeds this, the element fails constraint validation. If the value of the max attribute
        /// isn't a number, then the element has no maximum value.
        ///
        /// There is a special case: if the data type is periodic (such as for dates or times), the
        /// value of max may be lower than the value of min, which indicates that the range may wrap
        /// around; for example, this allows you to specify a time range from 10 PM to 4 AM.
        max

        /// Valid for text, search, url, tel, email, and password, it defines the maximum number of
        /// characters (as UTF-16 code units) the user can enter into the field. This must be an
        /// integer value 0 or higher. If no maxlength is specified, or an invalid value is
        /// specified, the field has no maximum length. This value must also be greater than or
        /// equal to the value of minlength.
        ///
        /// The input will fail constraint validation if the length of the text entered into the
        /// field is greater than maxlength UTF-16 code units long. By default, browsers prevent
        /// users from entering more characters than allowed by the maxlength attribute.
        maxlength

        /// Valid for date, month, week, time, datetime-local, number, and range, it defines the
        /// most negative value in the range of permitted values. If the value entered into the
        /// element is less than this this, the element fails constraint validation. If the value of
        /// the min attribute isn't a number, then the element has no minimum value.
        ///
        /// This value must be less than or equal to the value of the max attribute. If the min
        /// attribute is present but is not specified or is invalid, no min value is applied. If the
        /// min attribute is valid and a non-empty value is less than the minimum allowed by the min
        /// attribute, constraint validation will prevent form submission.
        ///
        /// There is a special case: if the data type is periodic (such as for dates or times), the
        /// value of max may be lower than the value of min, which indicates that the range may wrap
        /// around; for example, this allows you to specify a time range from 10 PM to 4 AM.
        min

        /// Valid for text, search, url, tel, email, and password, it defines the minimum number of
        /// characters (as UTF-16 code units) the user can enter into the entry field. This must be
        /// an non-negative integer value smaller than or equal to the value specified by maxlength.
        /// If no minlength is specified, or an invalid value is specified, the input has no minimum
        /// length.
        ///
        /// The input will fail constraint validation if the length of the text entered into the
        /// field is fewer than minlength UTF-16 code units long, preventing form submission.
        minlength

        /// If set, means the user can enter comma separated email addresses in the email widget or
        /// can choose more than one file with the file input. See the email and file input type.
        multiple(bool)

        /// A string specifying a name for the input control. This name is submitted along with the
        /// control's value when the form data is submitted.
        ///
        /// # What's in a name
        ///
        /// Consider the name a required attribute (even though it's not). If an input has no name
        /// specified, or name is empty, the input's value is not submitted with the form! (Disabled
        /// controls, unchecked radio buttons, unchecked checkboxes, and reset buttons are also not
        /// sent.)
        ///
        /// There are two special cases:
        ///
        /// * `_charset_`: If used as the name of an <input> element of type hidden, the input's
        ///   value is automatically set by the user agent to the character encoding being used to
        ///   submit the form.
        /// * `isindex`: For historical reasons, the name isindex is not allowed.
        ///
        /// # name and radio buttons
        ///
        /// The name attribute creates a unique behavior for radio buttons.
        ///
        /// Only one radio button in a same-named group of radio buttons can be checked at a time.
        /// Selecting any radio button in that group automatically deselects any currently-selected
        /// radio button in the same group. The value of that one checked radio button is sent along
        /// with the name if the form is submitted.
        ///
        /// When tabbing into a series of same-named group of radio buttons, if one is checked, that
        /// one will receive focus. If they aren't grouped together in source order, if one of the
        /// group is checked, tabbing into the group starts when the first one in the group is
        /// encountered, skipping all those that aren't checked. In other words, if one is checked,
        /// tabbing skips the unchecked radio buttons in the group. If none are checked, the radio
        /// button group receives focus when the first button in the same name group is reached.
        ///
        /// Once one of the radio buttons in a group has focus, using the arrow keys will navigate
        /// through all the radio buttons of the same name, even if the radio buttons are not
        /// grouped together in the source order.
        ///
        /// # HTMLFormElement.elements
        ///
        /// When an input element is given a name, that name becomes a property of the owning form
        /// element's HTMLFormElement.elements property.
        ///
        /// Warning: Avoid giving form elements a name that corresponds to a built-in property of
        /// the form, since you would then override the predefined property or method with this
        /// reference to the corresponding input.
        name

        /// The pattern attribute, when specified, is a regular expression that the input's value
        /// must match in order for the value to pass constraint validation. It must be a valid
        /// JavaScript regular expression, as used by the RegExp type, and as documented in our
        /// guide on regular expressions; the 'u' flag is specified when compiling the regular
        /// expression, so that the pattern is treated as a sequence of Unicode code points, instead
        /// of as ASCII. No forward slashes should be specified around the pattern text.
        ///
        /// If the pattern attribute is present but is not specified or is invalid, no regular
        /// expression is applied and this attribute is ignored completely. If the pattern attribute
        /// is valid and a non-empty value does not match the pattern, constraint validation will
        /// prevent form submission.
        ///
        /// Tip: If using the pattern attribute, inform the user about the expected format by
        /// including explanatory text nearby. You can also include a title attribute to explain
        /// what the requirements are to match the pattern; most browsers will display this title as
        /// a tooltip. The visible explanation is required for accessibility. The tooltip is an
        /// enhancement.
        pattern

        /// The placeholder attribute is a string that provides a brief hint to the user as to what
        /// kind of information is expected in the field. It should be a word or short phrase that
        /// demonstrates the expected type of data, rather than an explanatory message. The text
        /// must not include carriage returns or line feeds.
        ///
        /// Note: The placeholder attribute is not as semantically useful as other ways to explain
        /// your form, and can cause unexpected technical issues with your content.
        placeholder

        /// If present, indicates that the user should not be able to edit the value of the input.
        /// The readonly attribute is supported  text, search, url, tel, email, date, month, week,
        /// time, datetime-local, number, and password input types.
        readonly(bool)

        /// If present, indicates that the user must specify a value for the input before the owning
        /// form can be submitted. The required attribute is supported  text, search, url, tel,
        /// email, date, month, week, time, datetime-local, number, password, checkbox, radio, and
        /// file.
        required(bool)

        /// Valid for email, password, tel, and text input types only. Specifies how much of the
        /// input is shown. Basically creates same result as setting CSS width property with a few
        /// specialities. The actual unit of the value depends on the input type. For password and
        /// text it's number of characters (or em units) and pixels for others. CSS width takes
        /// precedence over size attribute.
        size

        /// Valid for the image input button only, the src is string specifying the URL of the image
        /// file to display to represent the graphical submit button. See the image input type.
        src

        /// Valid for the numeric input types, including number, date/time input types, and range,
        /// the step attribute is a number that specifies the granularity that the value must adhere
        /// to.
        ///
        /// If not explicitly included, step defaults to 1 for number and range, and 1 unit type
        /// (second, week, month, day) for the date/time input types. The value can must be a
        /// positive number—integer or float—or the special value any, which means no stepping is
        /// implied, and any value is allowed (barring other constraints, such as min and max).
        ///
        /// If any is not explicity set, valid values for the number, date/time input types, and
        /// range input types are equal to the basis for stepping - the min value and increments of
        /// the step value, up to the max value, if specified.
        ///
        /// For example, if you have <input type="number" min="10" step="2">, then any even integer,
        /// 10 or greater, is valid. If omitted, <input type="number">, any integer is valid, but
        /// floats (like 4.2) are not valid, because step defaults to 1. For 4.2 to be valid, step
        /// would have had to be set to any, 0.1, 0.2, or any the min value would have had to be a
        /// number ending in .2, such as <input type="number" min="-5.2">
        ///
        /// Note: When the data entered by the user doesn't adhere to the stepping configuration,
        /// the value is considered invalid in contraint validation and will match the :invalid
        /// pseudoclass.
        ///
        /// The default stepping value for number inputs is 1, allowing only integers to be entered,
        /// unless the stepping base is not an integer. The default stepping value for time is 1
        /// second (with 900 being equal to 15 minutes).
        step

        /// Global attribute valid for all elements, including all the input types, an integer
        /// attribute indicating if the element can take input focus (is focusable), if it should
        /// participate to sequential keyboard navigation. As all input types except for input of
        /// type hidden are focusable, this attribute should not be used on form controls, because
        /// doing so would require the management of the focus order for all elements within the
        /// document with the risk of harming usability and accessibility if done incorrectly.
        tabindex

        /// Global attribute valid for all elements, including all input types, containing a text
        /// representing advisory information related to the element it belongs to. Such information
        /// can typically, but not necessarily, be presented to the user as a tooltip. The title
        /// should NOT be used as the primary explanation of the purpose of the form control.
        /// Instead, use the <label> element with a for attribute set to the form control's id
        /// attribute.
        title

        /// A string specifying the type of control to render. For example, to create a checkbox, a
        /// value of checkbox is used. If omitted (or an unknown value is specified), the input type
        /// text is used, creating a plaintext input field.
        ///
        /// Permitted values are listed in <input> types above.
        type_

        /// The input control's value. When specified in the HTML, this is the initial value, and
        /// from then on it can be altered or retrieved at any time using JavaScript to access the
        /// respective HTMLInputElement object's value property. The value attribute is always
        /// optional, though should be considered mandatory for checkbox, radio, and hidden.
        value

        /// Valid for the image input button only, the width is the width of the image file to
        /// display to represent the graphical submit button. See the image input type.
        width
    }
}

html_element! {
    /// The [HTML `<label>` element][mdn] represents a caption for an item in a user interface.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label
    <label>

    attributes {
        /// The id of a labelable form-related element in the same document as the <label> element.
        /// The first element in the document with an id matching the value of the for attribute is
        /// the labeled control for this label element, if it is a labelable element. If it is not
        /// labelable then the for attribute has no effect. If there are other elements which also
        /// match the id value, later in the document, they are not considered.
        ///
        /// Note: A <label> element can have both a for attribute and a contained control element,
        /// as long as the for attribute points to the contained control element.
        for_

        /// The <form> element with which the label is associated (its form owner). If specified,
        /// the value of the attribute is the id of a <form> element in the same document. This lets
        /// you place label elements anywhere within a document, not just as descendants of their
        /// form elements.
        form
    }
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
    attributes {
        /// The current numeric value. This must be between the minimum and maximum values (min
        /// attribute and max attribute) if they are specified. If unspecified or malformed, the
        /// value is 0. If specified, but not within the range given by the min attribute and max
        /// attribute, the value is equal to the nearest end of the range.
        ///
        /// Note: Unless the value attribute is between 0 and 1 (inclusive), the min and max
        /// attributes should define the range so that the value attribute's value is within it.
        value

        /// The lower numeric bound of the measured range. This must be less than the maximum value
        /// (max attribute), if specified. If unspecified, the minimum value is 0.
        min

        /// The upper numeric bound of the measured range. This must be greater than the minimum
        /// value (min attribute), if specified. If unspecified, the maximum value is 1.
        max

        /// The <form> element to associate the <meter> element with (its form owner). The value of
        /// this attribute must be the id of a <form> in the same document. If this attribute is not
        /// set, the <button> is associated with its ancestor <form> element, if any. This attribute
        /// is only used if the <meter> element is being used as a form-associated element, such as
        /// one displaying a range corresponding to an <input type="number">.
        form

        /// The upper numeric bound of the low end of the measured range. This must be greater than
        /// the minimum value (min attribute), and it also must be less than the high value and
        /// maximum value (high attribute and max attribute, respectively), if any are specified. If
        /// unspecified, or if less than the minimum value, the low value is equal to the minimum
        /// value.
        high(u32)

        /// The lower numeric bound of the high end of the measured range. This must be less than
        /// the maximum value (max attribute), and it also must be greater than the low value and
        /// minimum value (low attribute and min attribute, respectively), if any are specified. If
        /// unspecified, or if greater than the maximum value, the high value is equal to the
        /// maximum value.
        low(u32)

        /// This attribute indicates the optimal numeric value. It must be within the range (as
        /// defined by the min attribute and max attribute). When used with the low attribute and
        /// high attribute, it gives an indication where along the range is considered preferable.
        /// For example, if it is between the min attribute and the low attribute, then the lower
        /// range is considered preferred.
        optimum(u32)
    }
}

html_element! {
    /// The [HTML `<optgroup>` element][mdn] creates a grouping of options within a
    /// [`<select>`][select] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/optgroup
    /// [select]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select
    <optgroup>

    attributes {
        /// If set, none of the items in this option group is selectable. Often browsers grey out
        /// such control and it won't receive any browsing events, like mouse clicks or
        /// focus-related ones.
        disabled(bool)

        /// The name of the group of options, which the browser can use when labeling the options in
        /// the user interface. This attribute is mandatory if this element is used.
        label
    }
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
    attributes {
        /// If set, this option is not checkable. Often browsers grey out such control and it won't
        /// receive any browsing event, like mouse clicks or focus-related ones. If this attribute
        /// is not set, the element can still be disabled if one of its ancestors is a disabled
        /// <optgroup> element.
        disabled(bool)

        /// This attribute is text for the label indicating the meaning of the option. If the label
        /// attribute isn't defined, its value is that of the element text content.
        label

        /// If present, indicates that the option is initially selected. If the <option> element is
        /// the descendant of a <select> element whose multiple attribute is not set, only one
        /// single <option> of this <select> element may have the selected attribute.
        selected(bool)

        /// The content of this attribute represents the value to be submitted with the form, should
        /// this option be selected. If this attribute is omitted, the value is taken from the text
        /// content of the option element.
        value
    }
}

html_element! {
    /// The [HTML Output element (`<output>`)][mdn] is a container element into which a site or app
    /// can inject the results of a calculation or the outcome of a user action.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/output
    <output>

    attributes {
        /// A space-separated list of other elements’ ids, indicating that those elements
        /// contributed input values to (or otherwise affected) the calculation.
        for_

        /// The <form> element to associate the output with (its form owner). The value of this
        /// attribute must be the id of a <form> in the same document. (If this attribute is not
        /// set, the <output> is associated with its ancestor <form> element, if any.)
        ///
        /// This attribute lets you associate <output> elements to <form>s anywhere in the document,
        /// not just inside a <form>. It can also override an ancestor <form> element.
        form

        /// The element's name. Used in the form.elements API.
        name
    }
}

html_element! {
    /// The [HTML `<progress>` element][progress] displays an indicator showing the completion
    /// progress of a task, typically displayed as a progress bar.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/progress
    <progress>

    attributes {
        /// This attribute describes how much work the task indicated by the progress element
        /// requires. The max attribute, if present, must have a value greater than 0 and be a valid
        /// floating point number. The default value is 1.
        max(f32)

        /// This attribute specifies how much of the task that has been completed. It must be a
        /// valid floating point number between 0 and max, or between 0 and 1 if max is omitted. If
        /// there is no value attribute, the progress bar is indeterminate; this indicates that an
        /// activity is ongoing with no indication of how long it is expected to take.
        value(f32)
    }
}

html_element! {
    /// The [HTML `<select>` element][mdn] represents a control that provides a menu of options.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select
    <select>
    attributes {
        /// A DOMString providing a hint for a user agent's autocomplete feature.
        autocomplete

        /// Lets you specify that a form control should have input focus when the page loads. Only
        /// one form element in a document can have the autofocus attribute.
        autofocus(bool)

        /// Indicates that the user cannot interact with the control. If this attribute is not
        /// specified, the control inherits its setting from the containing element, for example
        /// <fieldset>; if there is no containing element with the disabled attribute set, then the
        /// control is enabled.
        disabled(bool)

        /// The <form> element to associate the <select> with (its form owner). The value of this
        /// attribute must be the id of a <form> in the same document. (If this attribute is not
        /// set, the <select> is associated with its ancestor <form> element, if any.)
        ///
        /// This attribute lets you associate <select> elements to <form>s anywhere in the document,
        /// not just inside a <form>. It can also override an ancestor <form> element.
        form

        /// Indicates that multiple options can be selected in the list. If it is not specified,
        /// then only one option can be selected at a time. When multiple is specified, most
        /// browsers will show a scrolling list box instead of a single line dropdown.
        multiple(bool)

        /// This attribute is used to specify the name of the control.
        name

        /// Indicates that an option with a non-empty string value must be selected.
        required(bool)

        /// If the control is presented as a scrolling list box (e.g. when multiple is specified),
        /// this attribute represents the number of rows in the list that should be visible at one
        /// time. Browsers are not required to present a select element as a scrolled list box. The
        /// default value is 0.
        size
    }
}

html_element! {
    /// The [HTML `<textarea>` element][mdn] represents a multi-line plain-text editing control,
    /// useful when you want to allow users to enter a sizeable amount of free-form text, for
    /// example a comment on a review or feedback form.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/textarea
    <textarea>
    attributes {
        /// This attribute indicates whether the value of the control can be automatically completed
        /// by the browser. Possible values are:
        ///
        /// * off: The user must explicitly enter a value into this field for every use, or the
        ///   document provides its own auto-completion method; the browser does not automatically
        ///   complete the entry.
        /// * on: The browser can automatically complete the value based on values that the user has
        ///   entered during previous uses.
        ///
        /// If the autocomplete attribute is not specified on a <textarea> element, then the browser
        /// uses the autocomplete attribute value of the <textarea> element's form owner. The form
        /// owner is either the <form> element that this <textarea> element is a descendant of or
        /// the form element whose id is specified by the form attribute of the input element. For
        /// more information, see the autocomplete attribute in <form>.
        autocomplete

        /// Lets you specify that a form control should have input focus when the page loads. Only
        /// one form-associated element in a document can have this attribute specified.
        autofocus(bool)

        /// The visible width of the text control, in average character widths. If it is not
        /// specified, the default value is 20.
        cols(u32)

        /// Indicates that the user cannot interact with the control. If this attribute is not
        /// specified, the control inherits its setting from the containing element, for example
        /// <fieldset>; if there is no containing element when the disabled attribute is set, the
        /// control is enabled.
        disabled(bool)

        /// The form element that the <textarea> element is associated with (its "form owner"). The
        /// value of the attribute must be the id of a form element in the same document. If this
        /// attribute is not specified, the <textarea> element must be a descendant of a form
        /// element. This attribute enables you to place <textarea> elements anywhere within a
        /// document, not just as descendants of form elements.
        form

        /// The maximum number of characters (UTF-16 code units) that the user can enter. If this
        /// value isn't specified, the user can enter an unlimited number of characters.
        maxlength(u32)

        /// The minimum number of characters (UTF-16 code units) required that the user should
        /// enter.
        minlength(u32)

        /// The name of the control.
        name

        /// A hint to the user of what can be entered in the control. Carriage returns or line-feeds
        /// within the placeholder text must be treated as line breaks when rendering the hint.
        ///
        /// Note: Placeholders should only be used to show an example of the type of data that
        /// should be entered into a form; they are not a substitute for a proper <label> element
        /// tied to the input.
        placeholder

        /// Indicates that the user cannot modify the value of the control. Unlike the disabled
        /// attribute, the readonly attribute does not prevent the user from clicking or selecting
        /// in the control. The value of a read-only control is still submitted with the form.
        readonly(bool)

        /// This attribute specifies that the user must fill in a value before submitting a form.
        required

        /// The number of visible text lines for the control.
        rows

        /// Specifies whether the <textarea> is subject to spell checking by the underlying
        /// browser/OS. the value can be:
        ///
        /// * true: Indicates that the element needs to have its spelling and grammar checked.
        /// * default : Indicates that the element is to act according to a default behavior,
        ///   possibly based on the parent element's own spellcheck value.
        /// * false : Indicates that the element should not be spell checked.
        spellcheck

        /// Indicates how the control wraps text. Possible values are:
        ///
        /// * hard: The browser automatically inserts line breaks (CR+LF) so that each line has no
        ///   more than the width of the control; the cols attribute must also be specified for this
        ///   to take effect.
        /// * soft: The browser ensures that all line breaks in the value consist of a CR+LF pair,
        ///   but does not insert any additional line breaks.
        ///
        /// If this attribute is not specified, soft is its default value.
        wrap
    }
}
