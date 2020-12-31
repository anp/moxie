use mox::mox;
use moxie_dom::{
    elements::html::{button, li, ul},
    embed::DomLoop,
    prelude::*,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys as sys;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn mini_list() {
    let list = || {
        mox! {
            <ul class="listywisty">
                <li>"first"</li>
                <li class="item">"second"</li>
                <li>"third"</li>
            </ul>
        }
    };

    let web_div = augdom::document().create_element("div");
    let mut web_tester = DomLoop::new(web_div.clone(), list);
    let rsdom_root = augdom::create_virtual_element("div");
    let mut virtual_tester = DomLoop::new_virtual(rsdom_root.clone(), list);

    web_tester.run_once();
    virtual_tester.run_once();

    let expected_html = r#"<div><ul class="listywisty"><li>first</li><li class="item">second</li><li>third</li></ul></div>"#;

    assert_eq!(
        expected_html,
        web_div.outer_html(),
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
        &(String::from("\n") + &web_div.pretty_outer_html(2)),
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
    let web_div = augdom::document().create_element("div");
    let mut web_tester = DomLoop::new(web_div.clone(), move || {
        // Each event listener increments a counter
        let (counter1_val, counter1) = moxie::state(|| 0u8);
        let (counter2_val, counter2) = moxie::state(|| 0u8);

        let increment = |n: &u8| Some(n + 1);

        mox! {
            <button
                onclick={ move |_| counter1.update(increment) }
                onclick={ move |_| counter2.update(increment) }
            >
                // Display the values of the counters
                {format_args!("counter1 = {}, counter2 = {}", &counter1_val, &counter2_val)}
            </button>
        }
    });

    web_tester.run_once(); // Initial rendering

    // Retreive the HtmlElement of to the <button> tag
    let web_root_node: &sys::Node = web_div.expect_concrete();
    let web_root_element: &sys::Element = web_root_node.dyn_ref().unwrap();
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
