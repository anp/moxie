//! Trait for the base class of HTML elements.

use crate::interfaces::element::Element;

/// The HTMLElement interface represents any HTML element.
///
/// Note: this trait cannot be implemented outside of this crate.
pub trait HtmlElement: Element {
    attr_method! {
        /// Keyboard shortcut to activate or add focus to the element.
        accesskey
    }

    attr_method! {
        /// Sets whether input is automatically capitalized when entered by user. It can have the
        /// following values:
        ///
        /// * `off` or `none`, no autocapitalization is applied (all letters default to lowercase)
        /// * `on` or `sentences`, the first letter of each sentence defaults to a capital letter;
        ///   all other letters default to lowercase
        /// * `words`, the first letter of each word defaults to a capital letter; all other letters
        ///   default to lowercase
        /// * `characters`, all letters should default to uppercase
        autocapitalize
    }

    attr_method! {
        /// A value of "true" means the element is editable and a value of "false" means it isn't.
        contenteditable(bool)
    }

    /// Forms a class of attributes, called custom data attributes, that allow
    /// proprietary information to be exchanged between the HTML and its DOM
    /// representation that may be used by scripts.
    fn data(self, key: impl ToString, value: impl AsRef<str>) -> Self {
        self.attribute(&format!("data-{}", key.to_string()), value.as_ref())
    }

    attr_method! {
        /// The directionality of the element. It can have the following values:
        ///
        /// * `ltr`, which means left to right and is to be used for languages that are written from
        ///   the left to the right (like English);
        /// * `rtl`, which means right to left and is to be used for languages that are written from
        ///   the right to the left (like Arabic);
        /// * `auto`, which lets the user agent decide. It uses a basic algorithm as it parses the
        ///   characters inside the element until it finds a character with a strong directionality,
        ///   then it applies that directionality to the whole element.
        dir
    }

    attr_method! {
        /// Defines whether the element can be dragged.
        draggable(bool)
    }

    attr_method! {
        /// Indicates if the element is hidden or not.
        hidden(bool)
    }

    attr_method! {
        /// Indicates whether the user agent must act as though the given node is absent
        /// for the purposes of user interaction events, in-page text searches ("find in page"), and
        /// text selection.
        inert(bool)
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
