fn main() {
    println!("hi");
}

#[cfg(test)]
mod tests {
    use moxie_dom::{embed::WebRuntime, *};

    #[test]
    fn basic_list_prerender() {
        let (mut tester, root) = WebRuntime::in_rsdom_div(move || {
            moxie::mox! {
                <ul class="listywisty">
                    <li>"first"</li>
                    <li class="item">"second"</li>
                    <li>"third"</li>
                </ul>
            };
        });

        tester.run_once();
        let root = augdom::Node::Virtual(root);

        assert_eq!(
            &root.outer_html(),
            r#"<div><ul class="listywisty"><li>first</li><li class="item">second</li><li>third</li></ul></div>"#,
            "concisely-rendered string output must have no newlines or indentation"
        );

        assert_eq!(
            // this newline lets the below string output seem legible
            format!("\n{:#?}", &root),
            r#"
<div>
    <ul class="listywisty">
        <li>first</li>
        <li class="item">second</li>
        <li>third</li>
    </ul>
</div>"#,
            "pretty debug output must be 4-space-indented"
        );

        assert_eq!(
            // this newline lets the below string output seem legible
            format!("\n{}", &root),
            r#"
<div>
  <ul class="listywisty">
    <li>first</li>
    <li class="item">second</li>
    <li>third</li>
  </ul>
</div>"#,
            "Display output must be 2-space-indented"
        );
    }
}
