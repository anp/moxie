use {
    chashmap::CHashMap,
    once_cell::sync::Lazy,
    std::{
        any::{Any, TypeId},
        sync::Arc,
    },
    topo::{topo, Point},
};

/// Memoizes the provided function, invalidating previous memoizations if the argument has changed.
#[topo]
fn memo<Arg, Init, Output>(arg: Arg, initializer: Init) -> Output
where
    Arg: Clone + PartialEq + 'static,
    Init: FnOnce(Arg) -> Output,
    Output: Clone + Send + Sync + 'static,
{
    type Anon = Arc<dyn Any + Send + Sync>;
    static CALLSITES: Lazy<CHashMap<(TypeId, Point), Anon>> = Lazy::new(CHashMap::new);

    let key = (TypeId::of::<Output>(), Point::current());
    println!("memoizing {:?}", &key);

    let mut ret: Option<Output> = None;
    CALLSITES.alter(key, |maybe_val| {
        let val = if let Some(val) = maybe_val {
            val
        } else {
            Arc::new(initializer(arg))
        };

        let refd: &Output = val.downcast_ref().unwrap();
        ret = Some(refd.to_owned());

        Some(val)
    });

    ret.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[runtime::test]
    async fn basic_memo() {
        let mut call_count = 0;
        let mut call = || {
            memo!((), |()| {
                call_count += 1;
                call_count
            })
        };

        assert_eq!(call(), 1);
        assert_eq!(call(), 1);
        assert_eq!(call(), 1);
        assert_eq!(call(), 1);
    }
}
