//! Traits for nodes in the DOM tree.

/// This module is pub(crate) to ensure only the correct wrapper types access
/// untyped nodes via the traits defined here.
pub(crate) mod sealed {
    /// Implemented by types in this crate which wrap MemoNode.
    pub trait Memoized {
        /// Return a reference to the inner value.
        fn node(&self) -> &crate::memo_node::MemoNode;
    }
}

/// Node is an interface from which various types of DOM API objects inherit,
/// allowing those types to be treated similarly; for example, inheriting the
/// same set of methods, or being testable in the same way.
///
/// Note: this trait cannot be implemented outside of this crate.
pub trait Node: sealed::Memoized + Sized {
    /// The type returned by `build()`.
    type Output: Node;

    /// Retrieves access to the raw HTML element underlying the (MemoNode).
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

    /// Declare the element complete.
    fn build(self) -> Self::Output;
}

/// A node which accepts children.
pub trait Parent<Child: Node>: Node {
    /// Add a child to this node.
    fn child(self, child: Child) -> Self {
        let new_child = child.raw_node_that_has_sharp_edges_please_be_careful();
        self.node().ensure_child_attached(new_child);
        self
    }
}
