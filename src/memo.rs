use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

/// Memoizes the provided function, storing the intermediate `Init` value in memoization storage
/// and running `ret` with a reference to it, skipping the initialization on subsequent executions.
///
/// Marks the memoized value as `Live` in the current `Revision`.
#[topo::aware]
pub fn memo_with<Arg, Init, Ret>(
    arg: Arg,
    init: impl FnOnce(&Arg) -> Init,
    to_ret: impl FnOnce(&Init) -> Ret,
) -> Ret
where
    Arg: PartialEq + 'static,
    Init: 'static,
    Ret: 'static,
{
    let id = topo::Id::current();
    let store = topo::Env::expect::<MemoStore>();
    let mut store = store.0.borrow_mut();

    if let Some(memod) = store.get_if_arg_eq(id, &arg) {
        to_ret(memod)
    } else {
        let val = init(&arg);
        let returned = to_ret(&val);
        store.insert(id, arg, val);
        returned
    }
}

/// Memoizes the provided function once at the callsite. Runs `with` on every iteration.
#[topo::aware]
pub fn once_with<Out, Ret>(expr: impl FnOnce() -> Out, with: impl FnOnce(&Out) -> Ret) -> Ret
where
    Out: 'static,
    Ret: 'static,
{
    memo_with!((), |&()| expr(), with)
}

/// Memoize the provided function's output at this `topo::id`.
#[topo::aware]
pub fn memo<Arg, Out>(arg: Arg, init: impl FnOnce(&Arg) -> Out) -> Out
where
    Arg: PartialEq + 'static,
    Out: Clone + 'static,
{
    memo_with!(arg, init, Clone::clone)
}

/// Runs the provided expression once per [`topo::Id`], repeated calls at the same `Id` are assigned
/// adjacent slots. The provided value will always be cloned on subsequent calls unless dropped
/// from storage.
#[topo::aware]
pub fn once<Output>(expr: impl FnOnce() -> Output) -> Output
where
    Output: Clone + 'static,
{
    memo!((), |()| expr())
}

/// A shared pointer to the memoization storage singleton for a given runtime.
#[derive(Clone, Default)]
pub(crate) struct MemoStore(Rc<RefCell<MemoStorage>>);

impl MemoStore {
    /// Drops memoized values that were not referenced during the last tick and resets callsite
    /// repetition counts to 0.
    pub fn gc(&self) {
        self.0.borrow_mut().gc();
    }
}

/// The memoization storage for a `Runtime`. Stores memoized values by callsite, type, and a caller-
/// provided slot (a key for an internal hashmap)
/// exposing a garbage collection API to the embedding `Runtime`. Also tracks the number of times a
/// callsite has been invoked during a given `Revision`, allowing memoization to work without an
/// explicit slot.
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
    /// Retrieves a previously-initialized output for the requested callsite if the arguments
    /// are compatible. If a matching output is found the value is marked `Liveness::Live`.
    fn get_if_arg_eq<Arg, Out>(&mut self, id: topo::Id, arg: &Arg) -> Option<&Out>
    where
        Arg: PartialEq + 'static,
        Out: 'static,
    {
        if let Some((liveness, boxed)) =
            self.memos
                .get_mut(&(id, TypeId::of::<Arg>(), TypeId::of::<Out>()))
        {
            let (prev_arg, prev_out): &(Arg, Out) =
                boxed.downcast_ref().expect("looked up by type");
            if prev_arg == arg {
                *liveness = Liveness::Live;
                return Some(prev_out);
            }
        }

        None
    }

    /// Insert a new value into the memoization store.
    fn insert<Arg, Out>(&mut self, id: topo::Id, arg: Arg, val: Out)
    where
        Arg: PartialEq + 'static,
        Out: 'static,
    {
        self.memos.insert(
            (id, TypeId::of::<Arg>(), TypeId::of::<Out>()),
            (Liveness::Live, Box::new((arg, val))),
        );
    }

    /// Drops memoized values that were not referenced during the last tick, removing all `Dead`
    /// storage values and sets all remaining values to `Dead` for the next mark.
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
        crate::{memo::*, Revision},
        std::{cell::Cell, collections::HashSet},
    };

    fn with_test_logs(test: impl FnOnce()) {
        tracing::subscriber::with_default(
            tracing_fmt::FmtSubscriber::builder()
                .with_filter(tracing_fmt::filter::EnvFilter::new("warn"))
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
            let mut rt = crate::Runtime::new(|| {
                let revision = Revision::current();

                if let Some(pr) = prev_revision {
                    assert!(revision.0 > pr);
                } else {
                    comp_skipped_count += 1;
                }
                prev_revision = Some(revision.0);
                assert!(comp_skipped_count <= 1);

                assert!(revision.0 <= 5);
                let current_call_count = once!(|| {
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
        with_test_logs(|| {
            let mut ids = HashSet::new();
            for _ in 0..10 {
                topo::call!(ids.insert(topo::Id::current()));
            }
            assert_eq!(ids.len(), 10);

            let mut rt = crate::Runtime::new(|| {
                let mut ids = HashSet::new();
                for i in 0..10 {
                    memo!(i, |_| ids.insert(topo::Id::current()));
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
            let mut rt = crate::Runtime::new(|| {
                let mut counts = vec![];
                for i in 0..num_iters {
                    topo::call!(once!(|| counts.push(i)));
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
            let mut rt = crate::Runtime::new(|| {
                raw_exec.set(raw_exec.get() + 1);
                memo!(loop_ct.get(), |_| {
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
