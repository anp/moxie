//! moxie aims to empower everyone to build reliable and efficient human
//! interfaces. This crate implements a lightweight & platform-agnostic UI
//! runtime which powers a declarative style for creating interfaces and
//! attempts to minimize latency and general overhead.
//!
//! # Memoization
//!
//! Memoization is the core tool which moxie provides to store data across
//! `Revision`s and to minimize the incremental work performed when an update
//! triggers the next `Revision`. Calls to the memo_\* topological functions
//! will perform memoization specific to the current position within the
//! function call topology, as other topologically-nested functions do.
//!
//! During [run_once] the memoization storage is an  [environment
//! value](illicit::Env). Memoization calls write to this storage to store
//! results. At the end of [run_once], this storage is garbage-collected,
//! dropping values which were not referenced, marking them as live.
//!
//! Memoized values are dropped in a deterministic manner when replaced or no
//! longer referenced, modeling side-effectful lifecycle. Storing a type whose
//! liveness represents the effect being "active" allows us to perform the
//! effect when creating the stored value and to undo the effect when the stored
//! value is `Drop`ped.
//!
//! Initializing a memoized value at a particular callsite offers a simple API
//! for incremental computations. Further, placing mutations in the initializer
//! for a memo variable offers a path to minimizing the mutations or other side
//! effects performed while describing the interface.
//!
//! # State
//!
//! TODO(#95)
//!
//! # Loading
//!
//! TODO(#95)
//!
//! # UI Runtime
//!
//! A UI runtime is responsible for maintaining consistency between a program's
//! desired output and the rendered output over time. The desired output is
//! expected to change over time as a result of events from the "outside world."
//! This might be due to user input or events caused by asynchronous tasks
//! requested by a user.
//!
//! The rendered output is usually modelled or expressed in terms of a visual or
//! semantic hierarchy, or a tree. The relationships between elements in the
//! tree partition the space in which the elements are rendered, subdividing it
//! among their children. These relationships may or may not be concretely
//! encoded within data structures or they may purely be the result of some side
//! effects which occur in a particular order (e.g. mutating a display list for
//! a GPU).
//!
//! This process of performing the tasks to render the output is usually done in
//! a loop, iterating either once per fixed interval (e.g. 60 frames per second
//! or every 16.67 milliseconds) or when activated by the occurrence of events.
//!
//! [run_once]: crate::embed::Runtime::run_once
//! [topo]: https://docs.rs/topo

#![feature(track_caller)]
#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs)]

pub mod embed;
#[cfg(feature = "loading")]
pub mod load;
pub mod memo;
pub mod state;

/// A module for glob-importing the most commonly used moxie items.
pub mod prelude {
    pub use crate::{
        memo::{memo, memo_with, once, once_with},
        state::{memo_state, state, Key},
    };
    pub use topo;
}

// TODO(#115) add examples
/// Accepts an XML-like expression and expands it to builder method calls.
///
/// # Outputs
///
/// The `mox!` macro's contents are expanded to a nested builder pattern.
///
/// ## Tags
///
/// Each tag expands to a function call with the same name as the tag, with the
/// tag's arguments passed through as function arguments.
///
/// Each attribute expands to a method called on the value returned from the tag
/// opening. The attribute name is used as the method name, with the attribute
/// value passed as the argument.
///
/// A tag with children is treated as having an `inner` attribute to which a
/// closure is passed. This closure contains calls to the tag's children, in
/// order of declaration.
///
/// ## Fragments
///
/// Fragments expand to calls of each child tag in order of declaration.
///
/// ## Content
///
/// Content items are wrapped in calls to `text(...)`.
///
/// If a content item is a formatter the contained expression is first wrapped
/// in the `format!(...)` macro.
///
/// # Inputs
///
/// Each macro invocation must resolve to a single item. Items can be tags,
/// fragments, or content.
///
/// [snax](https://docs.rs/snax) is used to tokenize the input as [JSX](ish).
///
/// ## Tags
///
/// Tags always have a name and can have zero or more arguments, attributes, and
/// children.
///
/// They take the form `<NAME _=(ARGS ...) ATTR=VAL ...> CHILDREN </NAME>`.
/// Each optional portion can be omitted.
///
/// ### Arguments
///
/// A tag's arguments are wrapped with `_=(` and `)`, delimited by `,` (comma),
/// and must precede any attributes. Each argument is a Rust expression.
///
/// If there are no arguments the `_=()` wrapper must be omitted.
///
/// ### Attributes
///
/// Each attribute takes the form `NAME=VAL` where `NAME` is an identifier and
/// `VALUE` is an expression.
///
/// If the attribute's name is `type` it is rewritten as `ty` to avoid colliding
/// with the Rust keyword.
///
/// ### Children
///
/// Tags have zero or more nested items (tags, fragments, content) as children.
///
/// If there are no children the tag can be "self-closing": `<NAME ... />`.
///
/// ## Fragments
///
/// Fragments are opened with `<>` and closed with `</>`. Their only purpose is
/// to provide a parent for children. They do not accept arguments or
/// attributes.
///
/// ## Content
///
/// Content items represent text. They are delimited with `{` and `}`. They can
/// optionally be opened with `{%` to denote a "formatter" item.
///
/// [JSX]: https://facebook.github.io/jsx/
#[proc_macro_hack::proc_macro_hack(support_nested)]
pub use mox::mox;
