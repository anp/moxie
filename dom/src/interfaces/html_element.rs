//! Trait for the base class of HTML elements.

use crate::prelude::*;

/// The HTMLElement interface represents any HTML element.
///
/// Note: this trait cannot be implemented outside of this crate.
pub trait HtmlElement: Element {
    attr_method! {
        /// A value of "true" means the element is editable and a value of "false" means it isn't.
        content_editable
    }

    attr_method! {
        /// The directionality of the element. Possible values are "ltr", "rtl", and "auto".
        dir
    }

    attr_method! {
        /// A Boolean indicating if the element is hidden or not.
        hidden
    }

    attr_method! {
        /// A Boolean indicating whether the user agent must act as though the given node is absent
        /// for the purposes of user interaction events, in-page text searches ("find in page"), and
        /// text selection.
        inert
    }

    attr_method! {
        /// The language of an element's attributes, text, and element contents.
        lang
    }

    attr_method! {
        /// The text that appears in a popup box when mouse is over the element.
        title
    }
}
