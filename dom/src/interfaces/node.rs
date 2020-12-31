//! Traits for nodes in the DOM tree.

/// This module is pub(crate) to ensure only the correct wrapper types access
/// untyped nodes via the traits defined here.
pub(crate) mod sealed {
    /// Implemented by types in this crate which wrap CachedNode.
    pub trait Memoized {
        /// Return a reference to the inner value.
        fn node(&self) -> &crate::cached_node::CachedNode;
    }
}

/// Node is an interface from which various types of DOM API objects inherit,
/// allowing those types to be treated similarly; for example, inheriting the
/// same set of methods, or being testable in the same way.
///
/// Note: this trait cannot be implemented outside of this crate and is not
/// intended for direct use in most cases. See the
/// [`crate::interfaces::element`], module and its siblings, as well as the
/// [`Parent`] and [`Child`] traits in this module.
pub trait NodeWrapper: sealed::Memoized + Sized {
    /// Retrieves access to the raw HTML element underlying the (CachedNode).
    ///
    /// Because this offers an escape hatch around the memoized mutations, it
    /// should be used with caution. Also because of this, it has a silly
    /// name intended to loudly announce that care must be taken.
    ///
    /// Code called by the root function of your application will be run quite
    /// frequently and so the tools for memoization are important for
    /// keeping your application responsive. If you have legitimate needs
    /// for this API, please consider filing an issue with your use case so
    /// the maintainers of this crate can consider "official" ways to support
    /// it.
    fn raw_node_that_has_sharp_edges_please_be_careful(&self) -> &augdom::Node {
        self.node().raw_node()
    }
}

/// A value which can be bound as a child to a DOM node.
pub trait Child {
    /// Returns the "raw" node for this child to bind to its parent.
    fn to_bind(&self) -> &augdom::Node;

    /// Identity transform
    fn into_child(self) -> Self
    where
        Self: Sized,
    {
        self
    }
}

/// Convert `impl std::fmt::Display` into the `impl Child`
pub trait DisplayIntoChild: std::fmt::Display + Sized {
    /// Wrap `impl std::fmt::Display` into the `text` node
    fn into_child(self) -> crate::text::Text {
        // TODO rely on format_args, see [`(fmt_as_str #74442)`](https://github.com/rust-lang/rust/issues/74442)
        crate::text::text(format!("{}", self))
    }
}

impl<T> DisplayIntoChild for T where T: std::fmt::Display + Sized {}

impl<N> Child for N
where
    N: NodeWrapper,
{
    fn to_bind(&self) -> &augdom::Node {
        self.raw_node_that_has_sharp_edges_please_be_careful()
    }
}

/// A node which accepts children.
///
/// > Note: `C` is constrained by `Child` rather than `NodeWrapper` to allow
/// custom components to be bound directly to DOM types.
pub trait Parent<C: Child>: NodeWrapper {
    /// Add a child to this node.
    fn child(self, child: C) -> Self {
        self.node().ensure_child_attached(child.to_bind());
        self
    }
}
