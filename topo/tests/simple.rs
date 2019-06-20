use topo::*;

#[test]
fn invoke_test_topo() {
    #[bound]
    fn topo_test(prev: Id) {
        assert_ne!(prev, Id::current());
    }

    let prev = Id::current();
    topo_test!(prev);
}
