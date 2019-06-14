use {
    chashmap::CHashMap,
    once_cell::sync::Lazy,
    std::{
        any::{Any, TypeId},
        sync::Arc,
    },
    topo::{topo, Point},
};

/// Memoize the provided function at the bound callsite, invalidating previous memoizations if the
/// argument has changed.
#[topo]
pub fn memo<Arg, Init, Output>(arg: Arg, initializer: Init) -> Output
where
    Arg: PartialEq + Send + Sync + 'static,
    Output: Clone + Send + Sync + 'static,
    for<'a> Init: FnOnce(&'a Arg) -> Output,
{
    type Anon = Arc<dyn Any + Send + Sync>;
    static CALLSITES: Lazy<CHashMap<(TypeId, Point), Anon>> = Lazy::new(CHashMap::new);

    let key = (TypeId::of::<Output>(), Point::current());

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
    use crate::LoopBehavior;
    use {super::*, futures::FutureExt, std::panic::AssertUnwindSafe};

    #[runtime::test]
    async fn basic_memo() -> std::thread::Result<()> {
        AssertUnwindSafe(async {
            let mut call_count = 0u32;
            let mut tick_count = 0u32;

            println!("entering runloop");
            crate::runloop(|behavior| {
                tick_count += 1;

                assert!(tick_count <= 5);
                let current_call_count = memo!((), |()| {
                    println!("executing memo function");
                    call_count += 1;
                    call_count
                });

                assert_eq!(current_call_count, 1);
                assert_eq!(call_count, 1);
                if dbg!(tick_count) == 5 {
                    println!("stopping");
                    behavior.set(LoopBehavior::Stopped)
                } else {
                    println!("setting a keepalive");
                    behavior.set(LoopBehavior::Vsync(std::time::Duration::from_millis(16)))
                }
            })
            .await;

            assert_eq!(call_count, 1);
            assert_eq!(tick_count, 5);
        })
        .catch_unwind()
        .await
    }
}
