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
//! During [run_once] the memoization storage is an [environment
//! value](illicit::Layer). Memoization calls write to this storage to store
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
pub mod load;
pub mod memo;
pub mod state;

/// A module for glob-importing the most commonly used moxie items.
pub mod prelude {
    pub use crate::{memo, memo_state, memo_with, once, once_with, state, state::Key};
    pub use topo;
}

use embed::RunLoopWaker;
use memo::MemoStore;
use state::{Key, Var};

/// Memoizes the provided function, caching the intermediate `Stored` value in
/// memoization storage and only re-initializing it if `Arg` has changed since
/// the cached value was created. Regardless of prior cached results, `with` is
/// then called in to produce a return value.
///
/// Marks the memoized value as `Live` in the current `Revision`, preventing the
/// value from being garbage collected during at the end of the current
/// `Revision`.
///
/// If a previous value was cached for this callsite but the argument has
/// changed and it must be re-initialized, the previous value will be dropped
/// before the new one is initialized.
///
/// It is currently possible to nest calls to `memo_with` and other functions in
/// this module, but the values they store won't be correctly retained across
/// `Revision`s until we track dependency information. As a result, it's not
/// recommended to nest calls to `memo_with!`.
///
/// `init` takes a reference to `Arg` so that the memoization store can compare
/// future calls' arguments against the one used to produce the stored value.
#[topo::nested]
#[illicit::from_env(store: &MemoStore)]
pub fn memo_with<Arg, Stored, Ret>(
    arg: Arg,
    init: impl FnOnce(&Arg) -> Stored,
    with: impl FnOnce(&Stored) -> Ret,
) -> Ret
where
    Arg: PartialEq + 'static,
    Stored: 'static,
    Ret: 'static,
{
    store.memo_with(topo::Id::current(), arg, init, with)
}

/// Memoizes `expr` once at the callsite. Runs `with` on every iteration.
#[topo::nested]
#[illicit::from_env(store: &MemoStore)]
pub fn once_with<Stored, Ret>(
    expr: impl FnOnce() -> Stored,
    with: impl FnOnce(&Stored) -> Ret,
) -> Ret
where
    Stored: 'static,
    Ret: 'static,
{
    store.memo_with(topo::Id::current(), (), |&()| expr(), with)
}

/// Memoizes `init` at this callsite, cloning a cached `Stored` if it exists and
/// `Arg` is the same as when the stored value was created.
///
/// `init` takes a reference to `Arg` so that the memoization store can compare
/// future calls' arguments against the one used to produce the stored value.
#[topo::nested]
#[illicit::from_env(store: &MemoStore)]
pub fn memo<Arg, Stored>(arg: Arg, init: impl FnOnce(&Arg) -> Stored) -> Stored
where
    Arg: PartialEq + 'static,
    Stored: Clone + 'static,
{
    store.memo_with(topo::Id::current(), arg, init, Clone::clone)
}

/// Runs the provided expression once per [`topo::Id`]. The provided value will
/// always be cloned on subsequent calls unless dropped from storage and
/// reinitialized in a later `Revision`.
#[topo::nested]
#[illicit::from_env(store: &MemoStore)]
pub fn once<Stored>(expr: impl FnOnce() -> Stored) -> Stored
where
    Stored: Clone + 'static,
{
    store.memo_with(topo::Id::current(), (), |()| expr(), Clone::clone)
}

/// Root a state variable at this callsite, returning a [`Key`] to the state
/// variable.
#[topo::nested]
pub fn state<Init, Output>(init: Init) -> Key<Output>
where
    Output: 'static,
    Init: FnOnce() -> Output,
{
    memo_state((), |_| init())
}

/// Root a state variable at this callsite, returning a [`Key`] to the state
/// variable. Re-initializes the state variable if the capture `arg` changes.
#[topo::nested]
#[illicit::from_env(waker: &RunLoopWaker)]
pub fn memo_state<Arg, Init, Output>(arg: Arg, init: Init) -> Key<Output>
where
    Arg: PartialEq + 'static,
    Output: 'static,
    for<'a> Init: FnOnce(&'a Arg) -> Output,
{
    let var = memo(arg, |arg| Var::new(topo::Id::current(), waker.clone(), init(arg)));
    Var::root(var)
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
/// tag's arguments passed through as function arguments. The function call and
/// all attributes and children are wrapped in `#[topo::nested]` to create the
/// correct topology.
///
/// Each attribute expands to a method called on the value returned from the tag
/// opening or the previous attribute. The attribute name is used as the method
/// name, with the attribute value passed as the argument.
///
/// A tag with children has each child passed as the argument to a call to
/// `.child(...)`, one per child in order of declaration. The calls to `child`
/// come after attributes.
///
/// After all attributes and children, `.build()` is called on the final value.
///
/// ## Fragments
///
/// Fragments are not currently supported.
///
/// ## Content/Text
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
/// [snax](https://docs.rs/snax) is used to tokenize the input as [JSX]\(ish\).
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
/// If the attribute's name is `async`, `for`, `loop`, or `type` an underscore
/// is appended to avoid colliding with the Rust keyword.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::{Revision, Runtime};
    use std::{cell::Cell, collections::HashSet};

    fn with_test_logs(test: impl FnOnce()) {
        tracing::subscriber::with_default(
            tracing_subscriber::FmtSubscriber::builder()
                .with_env_filter(tracing_subscriber::filter::EnvFilter::new("warn"))
                .finish(),
            || {
                tracing::debug!("logging init'd");
                test();
            },
        );
    }

    #[test]
    fn basic_memo() {
        with_test_logs(|| {
            let mut call_count = 0u32;

            let mut prev_revision = None;
            let mut comp_skipped_count = 0;
            let mut rt = Runtime::new();
            let mut run = || {
                let revision = Revision::current();

                if let Some(pr) = prev_revision {
                    assert!(revision.0 > pr);
                } else {
                    comp_skipped_count += 1;
                }
                prev_revision = Some(revision.0);
                assert!(comp_skipped_count <= 1);

                assert!(revision.0 <= 5);
                let current_call_count = once(|| {
                    call_count += 1;
                    call_count
                });

                assert_eq!(current_call_count, 1);
                assert_eq!(call_count, 1);
            };

            for i in 0..5 {
                assert_eq!(rt.revision().0, i);

                rt.run_once(&mut run);

                assert_eq!(rt.revision().0, i + 1);
            }
            assert_eq!(call_count, 1);
        })
    }

    #[test]
    fn id_in_loop() {
        topo::call(|| {
            let mut ids = HashSet::new();
            for _ in 0..10 {
                topo::call(|| ids.insert(topo::Id::current()));
            }
            assert_eq!(ids.len(), 10);

            let mut rt = Runtime::new();
            rt.run_once(|| {
                let mut ids = HashSet::new();
                for i in 0..10 {
                    memo(i, |_| ids.insert(topo::Id::current()));
                }
                assert_eq!(ids.len(), 10);
            });
        });
    }

    #[test]
    fn memo_in_a_loop() {
        with_test_logs(|| {
            let num_iters = 10;
            let mut rt = Runtime::new();
            let run = || {
                let mut counts = vec![];
                for i in 0..num_iters {
                    topo::call(|| once(|| counts.push(i)));
                }
                counts
            };

            let first_counts = rt.run_once(run);
            assert_eq!(first_counts.len(), num_iters, "each mutation must be called exactly once");

            let second_counts = rt.run_once(run);
            assert_eq!(
                second_counts.len(),
                0,
                "each mutation was already called in the previous revision"
            );
        })
    }

    #[test]
    fn invalidation() {
        with_test_logs(|| {
            let loop_ct = Cell::new(0);
            let raw_exec = Cell::new(0);
            let memo_exec = Cell::new(0);
            let mut rt = Runtime::new();
            let run = || {
                raw_exec.set(raw_exec.get() + 1);
                memo(loop_ct.get(), |_| {
                    memo_exec.set(memo_exec.get() + 1);
                });
            };

            for i in 0..10 {
                loop_ct.set(i);

                assert_eq!(
                    memo_exec.get(),
                    i,
                    "memo block should execute exactly once per loop_ct value"
                );

                assert_eq!(
                    raw_exec.get(),
                    i * 2,
                    "runtime's root block should run exactly twice per loop_ct value"
                );

                rt.run_once(run);
                rt.run_once(run);
            }
        })
    }
}
