//! Element is the most general base class from which all element objects in a
//! Document inherit.

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
/// [HTMLElement]: [HtmlElementBuilder]
pub trait ElementBuilder: crate::interfaces::node::NodeWrapper {
    /// Declare an attribute of the element, mutating the actual element's
    /// attribute when the passed value changes.
    ///
    /// A guard value is stored as a resulting "effect" of the mutation, and
    /// removes the attribute when `drop`ped, to ensure that the attribute
    /// is removed when this declaration is no longer referenced in the most
    /// recent (`moxie::Revision`).
    fn attribute(self, name: &'static str, value: impl AsRef<str>) -> Self {
        self.node().set_attribute(name, value.as_ref());
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

/// A built Element. See [`ElementBuilder`] docs for more details.
pub trait Element: crate::interfaces::node::NodeWrapper {}
