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

        assert_eq!(
            &root.inner_html(),
            r#"<div><ul class="listywisty"><li>first</li><li class="item">second</li><li>third</li></ul></div>"#,
            "concisely-rendered string output must match expected"
        );
        assert_eq!(
            // this newline lets the above string output seem legible
            format!("\n{}", &root),
            r#"
<div>
    <ul class="listywisty">
        <li>first</li>
        <li class="item">second</li>
        <li>third</li>
    </ul>
</div>"#,
            "human-rendered string output must match expected"
        );
    }
}
