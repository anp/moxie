//! Trait for the base class of HTML elements.

use crate::prelude::*;

/// The HTMLElement interface represents any HTML element.
///
/// Note: this trait cannot be implemented outside of this crate.
pub trait HtmlElement: Element {
    attr_method! {
        /// Keyboard shortcut to activate or add focus to the element.
        accesskey
    }

    attr_method! {
        /// Sets whether input is automatically capitalized when entered by user.
        autocapitalize
    }

    bool_attr_method! {
        /// A value of "true" means the element is editable and a value of "false" means it isn't.
        contenteditable
    }

    /// Forms a class of attributes, called custom data attributes, that allow
    /// proprietary information to be exchanged between the HTML and its DOM
    /// representation that may be used by scripts.
    fn data(&self, key: impl ToString, value: impl ToString) -> &Self {
        self.attribute(&format!("data-{}", key.to_string()), value)
    }

    attr_method! {
        /// The directionality of the element. Possible values are "ltr", "rtl", and "auto".
        dir
    }

    bool_attr_method! {
        /// Defines whether the element can be dragged.
        draggable
    }

    bool_attr_method! {
        /// Indicates if the element is hidden or not.
        hidden
    }

    bool_attr_method! {
        /// Indicates whether the user agent must act as though the given node is absent
        /// for the purposes of user interaction events, in-page text searches ("find in page"), and
        /// text selection.
        inert
    }

    attr_method! {
        /// Provides a hint as to the type of data that might be entered by the user
        /// while editing the element or its contents. The attribute can be used with
        /// form controls (such as the value of textarea elements), or in elements in
        /// an editing host (e.g., using contenteditable attribute).
        inputmode
    }

    attr_method! {
        /// The language of an element's attributes, text, and element contents.
        lang
    }

    attr_method! {
        /// Assigns a slot in a shadow DOM shadow tree to an element.
        slot
    }

    attr_method! {
        /// Indicates whether spell checking is allowed for the element.
        spellcheck
    }

    attr_method! {
        /// Overrides the browser's default tab order and follows the one specified
        /// instead.
        tabindex
    }

    attr_method! {
        /// The text that appears in a popup box when mouse is over the element.
        title
    }
}
