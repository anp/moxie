#[test]
fn repro_syn2_upgrade_failure() {
    #[topo::nested(slot = name)]
    fn get_name_id(name: &str, _value: &str) -> topo::CallId {
        topo::CallId::current()
    }

    // reusing the same slot will get the same CallId
    let bob = get_name_id("bob", "hello");
    let bob_again = get_name_id("bob", "hello");
    assert_eq!(bob, bob_again);

    // the same name in a nested call returns a *new* CallId
    let bob_nested = topo::call(|| get_name_id("bob", "hello"));
    assert_ne!(bob, bob_nested);

    // different names produce different slots, even when other args are the same
    let alice_hello = get_name_id("alice", "hello");
    assert_ne!(bob, alice_hello);

    // changing non-slot arguments doesn't affect the CallId produced
    let alice_goodbye = get_name_id("alice", "goodbye");
    assert_eq!(alice_hello, alice_goodbye);
}
