use topo::*;

#[test]
fn one_child() {
    let root = Point::current();

    let second = __point!();
    assert_ne!(root, second);
    assert_eq!(root, Point::current());

    let mut called = false;
    second.enter(|| {
        assert_eq!(second, Point::current());
        called = true;
    });
    assert_eq!(root, Point::current());
    assert!(called);
}

#[test]
fn invoke_test_topo() {
    #[topo]
    fn topo_test(prev: Point) {
        assert_ne!(prev, Point::current());
    }

    // TODO(adam): move to an actual callstack and track the iteration count for the most recent
    // frame
    topo_test!(Point::current()); // this is analogous to the loops case actually, and is simpler
}

#[test]
#[ignore]
fn loops() {
    unimplemented!()
}

#[test]
fn parent_reset_on_recovered_panic() {
    let root = Point::current();

    let second = __point!();
    assert_ne!(root, second);
    assert_eq!(root, Point::current());

    let res = std::panic::catch_unwind(|| {
        second.enter(|| {
            assert_eq!(second, Point::current());
            call!(|| assert_ne!(Point::current(), second));
            panic!("the second should be unset by this");
        })
    });
    assert_eq!(root, Point::current());
    assert!(res.is_err());
}
