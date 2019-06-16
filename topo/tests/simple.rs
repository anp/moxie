use topo::*;

#[test]
fn invoke_test_topo() {
    #[topo]
    fn topo_test(prev: PointId) {
        assert_ne!(prev, PointId::current());
    }

    let prev = PointId::current();
    topo_test!(prev);
}
