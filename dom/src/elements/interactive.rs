//! HTML offers a selection of elements which help to create interactive user
//! interface objects.

html_element! {
    /// The [HTML Details Element (`<details>`)][mdn] creates a disclosure widget in which
    /// information is visible only when the widget is toggled into an "open" state.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details
    <details>

    attributes {
        /// Indicates whether the details will be shown on page load.
        open(bool)
    }
}

html_element! {
    /// The [HTML `<dialog>` element][mdn] represents a dialog box or other interactive component,
    /// such as an inspector or window.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dialog
    <dialog>

    attributes {
        /// Indicates that the dialog is active and can be interacted with. When the open attribute
        /// is not set, the dialog shouldn't be shown to the user.
        open(bool)
    }
}

html_element! {
    /// The [HTML `<menu>` element][mdn] represents a group of commands that a user can perform or
    /// activate. This includes both list menus, which might appear across the top of a screen, as
    /// well as context menus, such as those that might appear underneath a button after it has been
    /// clicked.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/menu
    <menu>
}

html_element! {
    /// The [HTML Disclosure Summary element (`<summary>`)][mdn] element specifies a summary,
    /// caption, or legend for a [`<details>`][details] element's disclosure box.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/summary
    /// [details]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details
    <summary>
}
