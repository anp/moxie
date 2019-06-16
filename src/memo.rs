use {
    chashmap::CHashMap,
    once_cell::sync::Lazy,
    std::{
        any::{Any, TypeId},
        sync::Arc,
    },
    topo::topo,
};

/// Memoize the provided function at the bound callsite, invalidating previous results only if
/// the explicitly passed argument has changed.
///
/// While we do have the option in Rust to compare the values of initializer closures we are passed,
/// it places a significant constraint on the initializers themselves to only capture `Clone` values
/// or to avoid mutating its captures to implement `Fn`. Instead we require that closures accept
/// the memoized argument by reference rather than by value.
#[topo]
pub fn memo<Arg, Init, Output>(arg: Arg, initializer: Init) -> Output
where
    Arg: PartialEq + Send + Sync + 'static,
    Output: Clone + Send + Sync + 'static,
    for<'a> Init: FnOnce(&'a Arg) -> Output,
{
    static CALLSITES: Lazy<CHashMap<(TypeId, topo::Id), Arc<dyn Any + Send + Sync>>> =
        Lazy::new(CHashMap::new);

    let key = (TypeId::of::<Output>(), topo::Id::current());

    let mut ret: Option<Output> = None;
    CALLSITES.alter(key, |maybe_val| {
        Some(
            maybe_val
                .and_then(|v| {
                    let (ref prev_arg, ref output): &(Arg, Output) = v.downcast_ref().unwrap();

                    if prev_arg == &arg {
                        ret = Some(output.to_owned());
                        Some(v)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| {
                    let output = initializer(&arg);
                    ret = Some(output.clone());
                    Arc::new((arg, output))
                }),
        )
    });
    ret.unwrap()
}

#[cfg(test)]
mod tests {
    use {
        crate::{memo::*, LoopBehavior, Revision},
        topo::__trace::*,
    };

    #[runtime::test]
    async fn basic_memo() {
        let mut call_count = 0u32;

        crate::runloop(|behavior| {
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
        })
        .await;

        assert_eq!(call_count, 1);
    }
}
