fn main() {
    println!("hi");
}

#[cfg(test)]
mod tests {
    use moxie_dom::{embed::WebRuntime, *};

    #[test]
    fn hello_world() {
        let (mut tester, root) = WebRuntime::with_rsdom(move || {
            moxie::mox! {
                <ul class="listywisty">
                    <li>"first"</li>
                    <li class="item">"second"</li>
                    <li>"third"</li>
                </ul>
            };
        });

        tester.run_once();

        let expected = r#"
<div>
  <ul class="listywisty">
    <li>first</li>
    <li class="item">second</li>
    <li>third</li>
  </ul>
</div>"#;

        assert_eq!(
            expected,
            // this newline lets the above string output seem legible
            String::from("\n") + &root.to_string(),
            "rendered string output must match expected"
        );
    }
}
