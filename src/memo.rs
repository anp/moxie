use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

/// Memoize the provided function's output at this `topo::id`.
#[topo::aware]
pub fn memo<Arg, Out>(arg: Arg, init: impl FnOnce(&Arg) -> Out) -> Out
where
    Arg: PartialEq + 'static,
    Out: Clone + 'static,
{
    let store = topo::Env::expect::<MemoStore>();
    store.get_or_init(arg, init)
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

    /// Returns a potentially memoized value for the given slot, argument, and initializer.
    ///
    /// This first checks for a matching value in storage, cloning it if available and returning
    /// that. If this callsite and slot haven't been previously initialized or if the argument
    /// previously used to do so is different from the current one, the initializer will be called
    /// again, that return value stored, and a clone returned to the caller.
    fn get_or_init<Arg, Out, Init>(&self, arg: Arg, initializer: Init) -> Out
    where
        Arg: PartialEq + 'static,
        Out: Clone + 'static,
        for<'a> Init: FnOnce(&'a Arg) -> Out,
    {
        let id = topo::Id::current();
        let maybe_memod = self.0.borrow_mut().get_if_arg_eq(id, &arg);
        // ^ this binding is necessary to keep the below borrow_mut from panicking
        maybe_memod.unwrap_or_else(|| {
            let new_output = initializer(&arg);
            self.0.borrow_mut().insert(id, arg, new_output.clone());
            new_output
        })
    }
}

/// The memoization storage for a `Runtime`. Stores memoized values by callsite, type, and a caller-
/// provided slot (a key for an internal hashmap)
/// exposing a garbage collection API to the embedding `Runtime`. Also tracks the number of times a
/// callsite has been invoked during a given `Revision`, allowing memoization to work without an
/// explicit slot.
pub(crate) struct MemoStorage {
    memos: HashMap<(topo::Id, TypeId, TypeId), (Liveness, Box<dyn Any>)>,
}

impl Default for MemoStorage {
    fn default() -> Self {
        MemoStorage {
            memos: HashMap::new(),
        }
    }
}

impl MemoStorage {
    /// Retrieves a previously-initialized output for the requested callsite if the arguments
    /// are compatible. If a matching output is found, cloned, and returned, the value is marked
    /// as `Liveness::Live`.
    fn get_if_arg_eq<Arg, Out>(&mut self, id: topo::Id, arg: &Arg) -> Option<Out>
    where
        Arg: PartialEq + 'static,
        Out: Clone + 'static,
    {
        if let Some((liveness, boxed)) =
            self.memos
                .get_mut(&(id, TypeId::of::<Arg>(), TypeId::of::<Out>()))
        {
            let (prev_arg, prev_out): &(Arg, Out) =
                boxed.downcast_ref().expect("looked up by type");
            if prev_arg == arg {
                *liveness = Liveness::Live;
                return Some(prev_out.clone());
            }
        }

        None
    }

    /// Insert a new value into the memoization store for a callsite under the given slot.
    fn insert<Arg, Out>(&mut self, id: topo::Id, arg: Arg, val: Out)
    where
        Arg: PartialEq + 'static,
        Out: Clone + 'static,
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
