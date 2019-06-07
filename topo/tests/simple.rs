use topo::*;

#[test]
fn invoke_test_topo() {
    #[topo]
    fn topo_test(prev: Point) {
        assert_ne!(prev, Point::current());
    }

    let prev = Point::current();
    topo_test!(prev);
}
