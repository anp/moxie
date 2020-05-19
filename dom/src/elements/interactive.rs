//! HTML offers a selection of elements which help to create interactive user
//! interface objects.

html_element! {
    /// The [HTML Details Element (`<details>`)][mdn] creates a disclosure widget in which
    /// information is visible only when the widget is toggled into an "open" state.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details
    <details>

    categories {
        Flow, Sectioning, Interactive, Palpable
    }

    children {
        tags {
            <summary>
        }
        categories {
            Flow
        }
    }

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

    categories {
        Flow, Sectioning
    }

    children {
        categories {
            Flow
        }
    }

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

    categories {
        Flow,
        Palpable // if the element's children include at least one <li> element
    }

    children {
        tags {
            // if in the list menu state
            <li>, <script>, <template>,

            // if in the context menu state
            <menu>, <menuitem>, <hr>, <script>, <template>
        }
        categories {
            // if in the list menu state
            Flow
        }
    }
}

html_element! {
    /// The [HTML Disclosure Summary element (`<summary>`)][mdn] element specifies a summary,
    /// caption, or legend for a [`<details>`][details] element's disclosure box.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/summary
    /// [details]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details
    <summary>

    children {
        categories {
            Phrasing, Heading
        }
    }
}
