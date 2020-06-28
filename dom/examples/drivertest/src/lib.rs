use moxie_dom::{
    elements::html::{button, li, ul},
    embed::WebRuntime,
    prelude::*,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys as sys;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn mini_list() {
    let list = || {
        moxie::mox! {
            <ul class="listywisty">
                <li>"first"</li>
                <li class="item">"second"</li>
                <li>"third"</li>
            </ul>
        }
    };

    let (mut web_tester, web_div) = WebRuntime::in_web_div(list);
    let (mut virtual_tester, rsdom_root) = WebRuntime::in_rsdom_div(list);
    let web_root_node: &sys::Node = web_div.as_ref();

    web_tester.run_once();
    virtual_tester.run_once();

    let expected_html = r#"<div><ul class="listywisty"><li>first</li><li class="item">second</li><li>third</li></ul></div>"#;

    assert_eq!(
        sys::Element::outer_html(&web_div),
        expected_html,
        "expected HTML must match the natively produced outer_html",
    );

    assert_eq!(
        expected_html,
        &augdom::Dom::outer_html(web_root_node),
        "our outer_html implementation must match the expected HTML",
    );

    assert_eq!(
        expected_html,
        rsdom_root.outer_html(),
        "HTML produced by virtual nodes must match expected",
    );

    let expected_pretty_html = r#"
<div>
  <ul class="listywisty">
    <li>first</li>
    <li class="item">second</li>
    <li>third</li>
  </ul>
</div>"#;

    assert_eq!(
        expected_pretty_html,
        &(String::from("\n") + &web_root_node.pretty_outer_html(2)),
        "pretty HTML produced from DOM nodes must match expected",
    );

    assert_eq!(
        expected_pretty_html,
        &(String::from("\n") + &rsdom_root.pretty_outer_html(2)),
        "pretty HTML produced from virtual nodes must match expected",
    );
}

#[wasm_bindgen_test]
fn mutiple_event_listeners() {
    // Create a button with two click event listeners
    let (mut web_tester, web_div) = WebRuntime::in_web_div(move || {
        // Each event listener increments a counter
        let counter1 = moxie::state(|| 0u8);
        let counter2 = moxie::state(|| 0u8);

        let increment = |n: &u8| Some(n + 1);

        // Clone the counters so they can be displayed later
        let (counter1_val, counter2_val) = (counter1.clone(), counter2.clone());

        moxie::mox! {
            <button
                onclick={ move |_| counter1.update(increment) }
                onclick={ move |_| counter2.update(increment) }
            >
                // Display the values of the counters
                {% "counter1 = {}, counter2 = {}", &counter1_val, &counter2_val }
            </button>
        }
    });

    web_tester.run_once(); // Initial rendering

    // Retreive the HtmlElement of to the <button> tag
    let web_root_element: &sys::Element = web_div.as_ref();
    let button_element = web_root_element
        .first_element_child()
        .and_then(|node| node.dyn_into::<sys::HtmlElement>().ok())
        .unwrap();

    assert_eq!(
        button_element.inner_text(),
        "counter1 = 0, counter2 = 0",
        "Counters should start at zero"
    );

    button_element.click(); // Simulate a click event
    web_tester.run_once(); // Update the DOM

    assert_eq!(
        button_element.inner_text(),
        "counter1 = 1, counter2 = 1",
        "Counters should be updated once"
    );
}
