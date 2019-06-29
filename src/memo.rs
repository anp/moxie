use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
};

/// Memoize the provided function at the bound callsite, invalidating previous results only if
/// the explicitly passed argument has changed.
///
/// While we do have the option in Rust to compare the values of initializer closures we are passed,
/// it places a significant constraint on the initializers themselves to only capture `Clone` values
/// or to avoid mutating its captures to implement `Fn`. Instead we require that closures accept
/// the memoized argument by reference rather than by value.
#[topo::bound]
pub fn memo<Arg, Init, Output>(arg: Arg, initializer: Init) -> Output
where
    Arg: PartialEq + 'static,
    Output: Clone + 'static,
    for<'a> Init: FnOnce(&'a Arg) -> Output,
{
    let callsites = topo::Env::expect::<MemoStore>();
    let callsites = &mut *(&*callsites).inner.borrow_mut();

    match callsites.entry((TypeId::of::<Output>(), topo::Id::current())) {
        Entry::Occupied(mut occ) => {
            let v = occ.get();
            let (ref prev_arg, ref output): &(Arg, Output) = v.downcast_ref().unwrap();

            if prev_arg == &arg {
                output.to_owned()
            } else {
                let new_output = initializer(&arg);
                occ.insert(Rc::new((arg, new_output.clone())));
                new_output
            }
        }
        Entry::Vacant(vac) => {
            let new_output = initializer(&arg);
            vac.insert(Rc::new((arg, new_output.clone())));
            new_output
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct MemoStore {
    inner: Rc<RefCell<HashMap<(TypeId, topo::Id), Rc<dyn Any>>>>,
}

#[cfg(test)]
mod tests {
    use crate::{memo::*, Revision};

    #[test]
    fn basic_memo() {
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
            let current_call_count = memo!((), |()| {
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
    }
}
