use {
    moxie_dom::{embed::WebRuntime, *},
    typed_html::dom::{DOMTree, VNode},
    wasm_bindgen::JsCast,
    wasm_bindgen_test::*,
};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn mini_list() {
    let body = document().body().unwrap();
    let root = document().create_element("div").unwrap();
    body.append_child(&root).unwrap();

    let mut expected: DOMTree<String> = typed_html::html!(
        <div>
            <ul>
                <li>"first"</li>
                <li>"second"</li>
                <li>"third"</li>
            </ul>
        </div>
    );

    let mut tester = WebRuntime::new(root.clone(), move || {
        moxie::mox! {
            <ul>
                <li>"first"</li>
                <li>"second"</li>
                <li>"third"</li>
            </ul>
        };
        assert_vnode_matches_element(&expected.vnode(), &root);
    });

    tester.run_once();
}

fn assert_vnode_matches_element(vnode: &VNode<String>, node: &sys::Node) {
    match (vnode, node.node_type()) {
        (VNode::Text(t), sys::Node::TEXT_NODE) => {
            assert_eq!(*t, &node.text_content().unwrap());
        }
        (VNode::Element(ve), sys::Node::ELEMENT_NODE) => {
            let elem: &sys::Element = node.dyn_ref().unwrap();
            assert_eq!(ve.name.to_lowercase(), node.node_name().to_lowercase());

            // for (name, value) in &e.attributes {
            //     // TODO make sure they're equal
            // }
            // // TODO make sure there aren't any missing or extras

            let mut actual_child = elem.first_child();
            for (i, expected_child) in ve.children.iter().enumerate() {
                let actual = match actual_child {
                    Some(a) => a,
                    None => panic!(
                        "failed while looking for child {} of {}",
                        i,
                        elem.inner_html()
                    ),
                };
                assert_vnode_matches_element(expected_child, &actual);
                actual_child = actual.next_sibling();
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
