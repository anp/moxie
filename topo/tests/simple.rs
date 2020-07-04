use topo::*;

#[test]
fn invoke_test_topo() {
    #[topo::nested]
    fn unique_id() -> CallId {
        CallId::current()
    }

    topo::call(|| {
        let mut prev = unique_id();
        for _ in 0..10 {
            let current = unique_id();
            assert_ne!(prev, current, "each CallId must be unique");
            prev = current;
        }
    });
}
