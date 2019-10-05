use {
    moxie_dom::{embed::WebRuntime, *},
    typed_html::dom::{DOMTree, VNode},
    wasm_bindgen::JsCast,
    wasm_bindgen_test::*,
};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn mini_list() {
    let mut expected: DOMTree<String> = typed_html::html!(
        <div>
            <ul>
                <li>"first"</li>
                <li>"second"</li>
                <li>"third"</li>
            </ul>
        </div>
    );
    let expected = expected.vnode();

    let root = document().create_element("div").unwrap();
    document().body().unwrap().append_child(&root).unwrap();

    let mut tester = WebRuntime::new(root.clone(), move || {
        moxie::mox! {
            <ul>
                <li>"first"</li>
                <li>"second"</li>
                <li>"third"</li>
            </ul>
        };
    });

    tester.run_once();
    assert_vnode_matches_element(&expected, &root);
}

fn assert_vnode_matches_element(expected: &VNode<String>, actual: &sys::Node) {
    match (expected, actual.node_type()) {
        (VNode::Text(expected), sys::Node::TEXT_NODE) => {
            assert_eq!(*expected, &actual.text_content().unwrap());
        }
        (VNode::Element(expected), sys::Node::ELEMENT_NODE) => {
            let actual: &sys::Element = actual.dyn_ref().unwrap();
            assert_eq!(
                expected.name.to_lowercase(),
                actual.node_name().to_lowercase(),
                "element types must match",
            );

            // for (name, value) in &e.attributes {
            //     // TODO make sure they're equal
            // }
            // // TODO make sure there aren't any missing or extras

            let mut actual_child = actual.first_child();
            for (i, expected_child) in expected.children.iter().enumerate() {
                let child = match actual_child {
                    Some(a) => a,
                    None => panic!(
                        "failed while looking for child {} of {}",
                        i,
                        actual.inner_html()
                    ),
                };
                assert_vnode_matches_element(expected_child, &child);
                actual_child = child.next_sibling();
            }
            assert!(
                actual_child.is_none(),
                "dom node should not have any children remaining"
            );
        }
        _ => {
            panic!("mismatched nodes!");
        }
    }
}
