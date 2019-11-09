//! Moxie implements "topological memoization" with storage in its runtime.
//!
//! TODO expand on this.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

/// Memoizes the provided function, caching the intermediate `Stored` value in memoization storage
/// and only re-initializing it if `Arg` has changed since the cached value was created. Regardless
/// of prior cached results, `with` is then called in to produce a return value.
///
/// Marks the memoized value as `Live` in the current `Revision`, preventing the value from being
/// garbage collected during at the end of the current `Revision`.
///
/// If a previous value was cached for this callsite but the argument has changed and it must be
/// re-initialized, the previous value will be dropped before the new one is initialized.
///
/// It is currently possible to nest calls to `memo_with` and other functions in this module, but
/// the values they store won't be correctly retained across `Revision`s until we track
/// dependency information. As a result, it's not recommended to nest calls to `memo_with!`.
///
/// `init` takes a reference to `Arg` so that the memoization store can compare future calls'
/// arguments against the one used to produce the stored value.
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
    let key = (
        topo::Id::current(),
        TypeId::of::<Arg>(),
        TypeId::of::<Stored>(),
    );

    // to allow nested memo_with calls, we separate mutable borrows of `store` from the callbacks:
    //
    // * with &mut store
    //   * remove optionally-cached value from storage
    // * call functions producing cached and returned values
    // * with &mut store
    //   * store cached value as Live, return returned value

    let stored = { store.0.borrow_mut().memos.remove(&key) };
    let mut with = Some(with); // wrapping in an option dodges the borrow checker for closures

    let mut cached = None;
    if let Some((_liveness, boxed)) = stored {
        let boxed: Box<(Arg, Stored)> = boxed.downcast().unwrap();

        if boxed.0 == arg {
            let with = with.take().unwrap();
            cached = Some((with(&boxed.1), boxed));
        } else {
            drop(boxed); // ensure that previous value is destroyed before init'ing another
        }
    };

    let (returned, boxed) = cached.unwrap_or_else(|| {
        let with = with.take().unwrap();
        let fresh = init(&arg);
        (with(&fresh), Box::new((arg, fresh)))
    });

    store
        .0
        .borrow_mut()
        .memos
        .insert(key, (Liveness::Live, boxed as _));

    returned
}

/// Memoizes `expr` once at the callsite. Runs `with` on every iteration.
#[topo::nested]
pub fn once_with<Stored, Ret>(
    expr: impl FnOnce() -> Stored,
    with: impl FnOnce(&Stored) -> Ret,
) -> Ret
where
    Stored: 'static,
    Ret: 'static,
{
    memo_with((), |&()| expr(), with)
}

/// Memoizes `init` at this callsite, cloning a cached `Stored` if it exists and `Arg` is the same
/// as when the stored value was created.
///
/// `init` takes a reference to `Arg` so that the memoization store can compare future calls'
/// arguments against the one used to produce the stored value.
#[topo::nested]
pub fn memo<Arg, Stored>(arg: Arg, init: impl FnOnce(&Arg) -> Stored) -> Stored
where
    Arg: PartialEq + 'static,
    Stored: Clone + 'static,
{
    memo_with(arg, init, Clone::clone)
}

/// Runs the provided expression once per [`topo::Id`]. The provided value will always be cloned on
/// subsequent calls unless dropped from storage and reinitialized in a later `Revision`.
#[topo::nested]
pub fn once<Stored>(expr: impl FnOnce() -> Stored) -> Stored
where
    Stored: Clone + 'static,
{
    memo((), |()| expr())
}

/// A shared pointer to the memoization storage singleton for a given runtime.
#[derive(Clone, Debug, Default)]
pub(crate) struct MemoStore(Rc<RefCell<MemoStorage>>);

impl MemoStore {
    /// Drops memoized values that were not referenced during the last `Revision`.
    pub fn gc(&self) {
        self.0.borrow_mut().gc();
    }
}

/// The memoization storage for a `Runtime`. Stores memoized values by a `MemoIndex`,
/// exposing a garbage collection API to the embedding `Runtime`.
#[derive(Debug)]
pub(crate) struct MemoStorage {
    memos: HashMap<MemoIndex, (Liveness, Box<dyn Any>)>,
}
type MemoIndex = (topo::Id, TypeId, TypeId);

impl Default for MemoStorage {
    fn default() -> Self {
        MemoStorage {
            memos: HashMap::new(),
        }
    }
}

impl MemoStorage {
    /// Drops memoized values that were not referenced during the last tick, removing all `Dead`
    /// storage values and sets all remaining values to `Dead` for the next GC execution.
    fn gc(&mut self) {
        self.memos
            .retain(|_, (liveness, _)| liveness == &Liveness::Live);
        self.memos
            .values_mut()
            .for_each(|(liveness, _)| *liveness = Liveness::Dead);
    }
}

/// Describes the outcome for a memoization value if a garbage collection were to occur when
/// observed. During the run of a `Revision` any memoized values which are initialized or read are
/// marked as `Live`. At the end of a `Revision`,
#[derive(Debug, PartialEq)]
enum Liveness {
    /// The memoized value would be retained in a GC right now.
    Live,
    /// The memoized value would be dropped in a GC right now.
    Dead,
}

#[cfg(test)]
mod tests {
    use {
        crate::{
            embed::{Revision, Runtime},
            memo::*,
        },
        std::{cell::Cell, collections::HashSet},
    };

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
            let mut rt = Runtime::new(|| {
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
            });

            for i in 0..5 {
                assert_eq!(rt.revision().0, i);

                rt.run_once();

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

            let mut rt = Runtime::new(|| {
                let mut ids = HashSet::new();
                for i in 0..10 {
                    memo(i, |_| ids.insert(topo::Id::current()));
                }
                assert_eq!(ids.len(), 10);
            });
            rt.run_once();
        });
    }

    #[test]
    fn memo_in_a_loop() {
        with_test_logs(|| {
            let num_iters = 10;
            let mut rt = Runtime::new(|| {
                let mut counts = vec![];
                for i in 0..num_iters {
                    topo::call(|| once(|| counts.push(i)));
                }
                counts
            });

            let first_counts = rt.run_once();
            assert_eq!(
                first_counts.len(),
                num_iters,
                "each mutation must be called exactly once"
            );

            let second_counts = rt.run_once();
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
            let mut rt = Runtime::new(|| {
                raw_exec.set(raw_exec.get() + 1);
                memo(loop_ct.get(), |_| {
                    memo_exec.set(memo_exec.get() + 1);
                });
            });

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

                rt.run_once();
                rt.run_once();
            }
        })
    }
}
