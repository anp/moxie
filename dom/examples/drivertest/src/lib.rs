use {
    moxie_dom::{embed::WebRuntime, *},
    std::io::prelude::*,
    typed_html::dom::{DOMTree, VNode},
    wasm_bindgen::JsCast,
    wasm_bindgen_test::*,
    web_sys as sys,
};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn mini_list() {
    let mut expected: DOMTree<String> = typed_html::html!(
        <div>
            <ul class="listywisty">
                <li>"first"</li>
                <li class="item">"second"</li>
                <li>"third"</li>
            </ul>
        </div>
    );
    let expected = expected.vnode();

    let list = || {
        moxie::mox! {
            <ul class="listywisty">
                <li>"first"</li>
                <li class="item">"second"</li>
                <li>"third"</li>
            </ul>
        };
    };

    let (mut web_tester, web_root) = WebRuntime::in_web_div(list);
    let (mut virtual_tester, rsdom_root) = WebRuntime::in_rsdom_div(list);

    web_tester.run_once();
    virtual_tester.run_once();

    assert_vnode_matches_element(&expected, &web_root);

    let expected_html =
        r#"<div><ul class="listywisty"><li>first</li><li class="item">second</li><li>third</li></ul></div>"#;

    assert_eq!(
        expected_html,
        web_root.outer_html(),
        "HTML produced by DOM nodes must match expected",
    );

    assert_eq!(
        expected_html,
        rsdom_root.outer_html(),
        "HTML produced by virtual nodes must match expected",
    );
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

            assert_attributes_match(expected, actual);

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

fn assert_attributes_match(expected: &typed_html::dom::VElement<String>, actual: &sys::Element) {
    let mut attr_panic_msg = Vec::new();

    let mut expected_attrs = std::collections::BTreeMap::new();
    for (name, value) in &expected.attributes {
        expected_attrs.insert(name.to_string(), value);
    }

    let actual_attrs = actual.attributes();
    for i in 0..actual_attrs.length() {
        let actual = actual_attrs.item(i).unwrap();
        let name = actual.local_name();
        let expected = expected_attrs.remove(&name);

        if let Some(expected) = expected {
            assert_eq!(&actual.value(), expected, "attribute `{}` must match", name);
        } else {
            writeln!(
                &mut attr_panic_msg,
                "unexpected {}={}",
                name,
                actual.value()
            )
            .unwrap();
        }
    }

    for (expected_name, expected_value) in expected_attrs {
        writeln!(
            &mut attr_panic_msg,
            "missing {}={}",
            expected_name, expected_value
        )
        .unwrap();
    }

    if !attr_panic_msg.is_empty() {
        let msg = String::from_utf8(attr_panic_msg).unwrap();
        panic!("attributes mismatched:\n{}", msg);
    }
}
