//! Element is the most general base class from which all element objects in a
//! Document inherit.

use crate::prelude::*;

/// Element is the most general base class from which all element objects (i.e.
/// objects that represent elements) in a Document inherit. It only has methods
/// and properties common to all kinds of elements. More specific classes
/// inherit from Element. For example, the [HTMLElement] interface is the base
/// interface for HTML elements, while the SVGElement interface is the basis for
/// all SVG elements. Most functionality is specified further down the class
/// hierarchy.
///
/// Note: this trait cannot be implemented outside of this crate.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Element
/// [HTMLElement]: [HtmlElement]
pub trait Element: Node {
    /// Declare an attribute of the element, mutating the actual element's
    /// attribute when the passed value changes.
    ///
    /// A guard value is stored as a resulting "effect" of the mutation, and
    /// removes the attribute when `drop`ped, to ensure that the attribute
    /// is removed when this declaration is no longer referenced in the most
    /// recent (`moxie::Revision`).
    #[topo::nested]
    fn attribute(&self, #[slot] name: &str, value: impl ToString) -> &Self {
        self.node().memo_attribute(name, value.to_string());
        self
    }

    attr_method! {
        /// Updates the element's `class`.
        class
    }

    attr_method! {
        /// Updates the element's `id`.
        id
    }

    attr_method! {
        /// Updates the element's [`style`].
        ///
        /// [`style`]: https://developer.mozilla.org/en-US/docs/Web/API/ElementCSSInlineStyle/style
        style
    }
}
