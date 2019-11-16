//! moxie aims to empower everyone to build reliable and efficient human interfaces. This crate
//! implements a lightweight & platform-agnostic UI runtime which powers a declarative style for
//! creating interfaces and attempts to minimize latency and general overhead.
//!
//! TODO simple snippet
//!
//! # Declarative style
//!
//! TODO "imperative but idempotent"
//!
//! "describe the UI *right now*"
//!
//! partition the render space using function calls
//!
//! while managing persistent stateful elements
//!
//! with minimal incremental updates
//!
//! in order to achieve minimal latency and consistent responsiveness
//!
//! # Memoization
//!
//! Memoization is the core tool which moxie provides to store data across `Revision`s and to
//! minimize the incremental work performed when an update triggers the next `Revision`. Calls to
//! the memo_\* topological functions will perform memoization specific to the current position
//! within the function call topology, as other topologically-nested functions do.
//!
//! During [`run_once`] the memoization storage is an [environment value](topo::Env). Memoization
//! calls write to this storage to store results. At the end of [run_once], this storage is
//! garbage-collected, dropping values which were not referenced, marking them as live.
//!
//! Memoized values are dropped in a deterministic manner when replaced or no longer referenced,
//! modeling side-effectful lifecycle. Storing a type whose liveness represents the effect being
//! "active" allows us to perform the effect when creating the stored value and to undo the effect
//! when the stored value is `Drop`ped.
//!
//! Initializing a memoized value at a particular callsite offers a simple API for incremental
//! computations. Further, placing mutations in the initializer for a memo variable offers a path
//! to minimizing the mutations or other side effects performed while describing the interface.
//!
//! # State
//!
//! TODO
//!
//! # UI Runtime
//!
//! A UI runtime is responsible for maintaining consistency between a program's desired output and
//! the rendered output over time. The desired output is expected to change over time as a result
//! of events from the "outside world." This might be due to user input or events caused by
//! asynchronous tasks requested by a user.
//!
//! The rendered output is usually modelled or expressed in terms of a visual or semantic hierarchy,
//! or a tree. The relationships between elements in the tree partition the space in which the
//! elements are rendered, subdividing it among their children. These relationships may or may
//! not be concretely encoded within data structures or they may purely be the result of some side
//! effects which occur in a particular order (e.g. mutating a display list for a GPU).
//!
//! This process of performing the tasks to render the output is usually done in a loop, iterating
//! either once per fixed interval (e.g. 60 frames per second or every 16.67 milliseconds) or when
//! activated by the occurrence of events.
//!
//! [run_once]: crate::embed::Runtime::run_once
//! [topo]: https://docs.rs/topo

#![forbid(unsafe_code)]
#![deny(clippy::all, intra_doc_link_resolution_failure, missing_docs)]

/// TODO figure out where this is actually documented
#[proc_macro_hack::proc_macro_hack(support_nested)]
pub use mox::mox;

pub mod embed;
#[macro_use]
mod memo;
#[macro_use]
mod state;
mod load;

pub use topo;
#[doc(inline)]
pub use {load::*, memo::*, state::*};
