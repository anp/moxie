use {
    downcast_rs::{impl_downcast, Downcast},
    std::{any::TypeId, cell::RefCell, collections::HashMap, hash::Hash, rc::Rc},
};

/// Memoize the provided function at this `topo::id`, using an iteration counter as the
/// memoization slot (see [`memo_by_slot`] for details). The counter is incremented once
/// for each call at the same callsite in a given [`state::Revision`].
#[topo::bound]
pub fn memo<Arg, Out>(arg: Arg, init: impl FnOnce(&Arg) -> Out) -> Out
where
    Arg: PartialEq + 'static,
    Out: Clone + 'static,
{
    let store = topo::Env::expect::<MemoStore>();
    let slot = store.next_slot_index::<Arg, Out>(topo::Id::current());
    store.get_or_init(slot, arg, init)
}

/// Runs the provided expression once per [`topo::Id`], repeated calls at the same `Id` are assigned
/// adjacent slots. The provided value will always be cloned on subsequent calls unless dropped
/// from storage.
#[topo::bound]
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

    /// Returns the next iteration-order slot for the passed callsite during this `Revision`.
    /// Advances the iteration order counter for the passed callsite as well.
    fn next_slot_index<Arg, Out>(&self, id: topo::Id) -> u32
    where
        Arg: PartialEq + 'static,
        Out: 'static,
    {
        self.0.borrow_mut().next_slot_index(id)
    }

    /// Returns a potentially memoized value for the given slot, argument, and initializer.
    ///
    /// This first checks for a matching value in storage, cloning it if available and returning
    /// that. If this callsite and slot haven't been previously initialized or if the argument
    /// previously used to do so is different from the current one, the initializer will be called
    /// again, that return value stored, and a clone returned to the caller.
    fn get_or_init<Slot, Arg, Out, Init>(&self, slot: Slot, arg: Arg, initializer: Init) -> Out
    where
        Slot: Eq + Hash + 'static,
        Arg: PartialEq + 'static,
        Out: Clone + 'static,
        for<'a> Init: FnOnce(&'a Arg) -> Out,
    {
        let id = topo::Id::current();
        let maybe_memod = self
            .0
            .borrow_mut()
            .with_callsite_storage(id, |storage: &mut CallsiteStorage<Slot, Arg, Out>| {
                storage.get_if_arg_eq(&slot, &arg)
            });
        // ^ this binding is necessary to keep the below borrow_mut from panicking
        maybe_memod.unwrap_or_else(|| {
            let new_output = initializer(&arg);
            self.0
                .borrow_mut()
                .insert(id, slot, arg, new_output.clone());
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
    memos: HashMap<(topo::Id, TypeId, TypeId, TypeId), Box<dyn Gc>>,
    default_slots: HashMap<topo::Id, u32>,
}

impl Default for MemoStorage {
    fn default() -> Self {
        MemoStorage {
            memos: HashMap::new(),
            default_slots: HashMap::new(),
        }
    }
}

impl MemoStorage {
    /// Insert a new value into the memoization store for a callsite under the given slot.
    fn insert<Slot: Eq + Hash, Arg: 'static, Out: 'static>(
        &mut self,
        id: topo::Id,
        slot: Slot,
        arg: Arg,
        val: Out,
    ) where
        Slot: Eq + Hash + 'static,
        Arg: PartialEq + 'static,
        Out: Clone + 'static,
    {
        self.with_callsite_storage(id, move |storage| {
            storage.inner.insert(slot, (Liveness::Live, arg, val))
        });
    }

    /// Called to provide unique slot values when e.g. loop iteration order is the only slot
    /// available.
    fn next_slot_index(&mut self, id: topo::Id) -> u32 {
        let current = self.default_slots.entry(id).or_default();
        *current += 1;
        *current
    }

    /// Erases the previous tick's callsite iteration counts and removes all `Dead` storage values.
    fn gc(&mut self) {
        self.default_slots.clear();
        self.memos.values_mut().for_each(|store| store.gc());
    }

    /// Runs the provided closure with typed mutable access to the [`CallsiteStorage`] for the
    /// passed [`topo::Id`].
    fn with_callsite_storage<Slot, Arg, Out, Ret>(
        &mut self,
        id: topo::Id,
        op: impl FnOnce(&mut CallsiteStorage<Slot, Arg, Out>) -> Ret,
    ) -> Ret
    where
        Slot: Eq + Hash + 'static,
        Arg: PartialEq + 'static,
        Out: Clone + 'static,
    {
        #[allow(clippy::borrowed_box)]
        let storage: &mut Box<dyn Gc> = self
            .memos
            .entry((
                id,
                TypeId::of::<Slot>(),
                TypeId::of::<Arg>(),
                TypeId::of::<Out>(),
            ))
            .or_insert_with(CallsiteStorage::<Slot, Arg, Out>::boxed);
        let storage: &mut CallsiteStorage<Slot, Arg, Out> = storage.downcast_mut().unwrap();
        op(storage)
    }
}

/// Stores memoized values and their arguments for a given [`topo::Id`]. Storage is indexed by
/// the memoization slot, allowing multiple memoization values to reside under the same callsite's
/// storage, e.g. for values memoized in a loop.
struct CallsiteStorage<Slot, Arg, Out>
where
    Slot: Eq + Hash,
{
    inner: HashMap<Slot, (Liveness, Arg, Out)>,
}

impl<Slot, Arg, Out> CallsiteStorage<Slot, Arg, Out>
where
    Slot: Eq + Hash + 'static,
    Arg: PartialEq + 'static,
    Out: Clone + 'static,
{
    fn boxed() -> Box<dyn Gc> {
        Box::new(Self {
            inner: Default::default(),
        })
    }

    /// Returns an owned copy of the previously-initialized output if it exists under the provided
    /// slot and the argument used to initialize it is the same as current one.
    fn get_if_arg_eq(&mut self, slot: &Slot, arg: &Arg) -> Option<Out> {
        if let Some((liveness, prev_arg, prev_output)) = self.inner.get_mut(slot) {
            if arg == prev_arg {
                *liveness = Liveness::Live;
                return Some(prev_output.clone());
            }
        }
        None
    }
}

impl<Slot, Arg, Out> Gc for CallsiteStorage<Slot, Arg, Out>
where
    Slot: Eq + Hash + 'static,
    Arg: 'static,
    Out: 'static,
{
    /// The garbage collection scheme implemented here has roughly three phases:
    ///
    /// 1. Before this method is called, memoized values are marked as `Live` when read or created.
    /// 2. This method is called and only `Live` values are retained, droppping `Dead` values.
    /// 3. All remaining `Live` values are marked `Dead`. This method exits.
    fn gc(&mut self) {
        self.inner
            .retain(|_, (liveness, _, _)| liveness == &Liveness::Live);
        self.inner
            .values_mut()
            .for_each(|(liveness, _, _)| *liveness = Liveness::Dead);
    }
}

/// An object-safe trait that allows us to store disjoint types for many callsites while also
/// running a single GC pass at the end of a `Revision`, while also safely casting the boxed
/// storage to the underlying concrete type during a `Revision`.
trait Gc: Downcast {
    /// Drop any unreferenced values.
    fn gc(&mut self);
}
impl_downcast!(Gc);

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
        std::cell::Cell,
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
