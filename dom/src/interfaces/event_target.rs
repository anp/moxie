//! EventTarget is a DOM interface implemented by objects that can receive
//! events and may have listeners for them.

use augdom::event::{Event, EventHandle};

use crate::prelude::*;

/// EventTarget is a DOM interface implemented by objects that can receive
/// events and may have listeners for them.
///
/// Element, Document, and Window are the most common event targets, but other
/// objects can be event targets, too. For example XMLHttpRequest, AudioNode,
/// AudioContext, and others.
///
/// Many event targets (including elements, documents, and windows) also support
/// setting event handlers via onevent properties and attributes.
///
/// Note: this trait cannot be implemented outside of this crate.
pub trait EventTarget<Ev>: Node
where
    Ev: 'static + Event,
{
    /// Declare an event handler on the element.
    ///
    /// A guard value is stored as a resulting "effect" of the mutation, and
    /// removes the attribute when `drop`ped, to ensure that the attribute
    /// is removed when this declaration is no longer referenced in the most
    /// recent (`moxie::Revision`).
    ///
    /// Currently this is performed on every Revision, as changes to event
    /// handlers don't typically affect the debugging experience and have
    /// not yet shown up in performance profiles.
    #[topo::nested]
    fn on(self, callback: impl FnMut(Ev) + 'static) -> Self {
        memo_with(
            moxie::embed::Revision::current(),
            |_| EventHandle::new(self.raw_node_that_has_sharp_edges_please_be_careful(), callback),
            |_| {},
        );
        self
    }
}
