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
    let callsites = topo::Env::get::<MemoStore>().unwrap();
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
    use {
        crate::{memo::*, LoopBehavior, Revision},
        futures::executor::block_on,
        tokio_trace::*,
    };

    #[test]
    fn basic_memo() {
        let mut call_count = 0u32;

        block_on(crate::runloop(|behavior| {
            let revision = Revision::current();

            assert!(revision.0 <= 5);
            let current_call_count = memo!((), |()| {
                info!("executing memo function");
                call_count += 1;
                call_count
            });

            assert_eq!(current_call_count, 1);
            assert_eq!(call_count, 1);
            if revision.0 == 5 {
                info!("stopping");
                behavior.set(LoopBehavior::Stopped);
            } else {
                behavior.set(LoopBehavior::Continue);
            }
        }));

        assert_eq!(call_count, 1);
    }
}
