//! HTML offers a selection of elements which help to create interactive user
//! interface objects.

use crate::{
    interfaces::node::{sealed::Memoized, Node},
    memo_node::MemoNode,
    prelude::*,
};
use augdom::event;

html_element! {
    /// The [HTML Details Element (`<details>`)][mdn] creates a disclosure widget in which
    /// information is visible only when the widget is toggled into an "open" state.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details
    <details>
}

impl Details {
    attr_method! {
        /// Indicates whether the details will be shown on page load.
        pub open(bool)
    }
}

html_element! {
    /// The [HTML `<dialog>` element][mdn] represents a dialog box or other interactive component,
    /// such as an inspector or window.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dialog
    <dialog>
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
