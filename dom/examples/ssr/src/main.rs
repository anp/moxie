#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate serde_derive;

use {
    gotham::router::builder::*,
    gotham::router::Router,
    gotham::state::{FromState, State},
    moxie_dom::{
        elements::{li, ul},
        prelude::*,
    },
};

fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct PathExtractor {
    #[serde(rename = "*")]
    parts: Vec<String>,
}

#[topo::nested]
fn simple_list(items: &[String]) {
    moxie::mox! {
        <ul>{
            for item in items {
                moxie::mox!(<li>{% "{}", item }</li>)
            }
        }</ul>
    }
}

fn parts_handler(state: State) -> (State, String) {
    let parts = {
        let path = PathExtractor::borrow_from(&state);
        path.parts.to_owned()
    };
    let res = moxie_dom::render_html(move || simple_list(&parts));
    (state, res)
}

fn router() -> Router {
    build_simple_router(|route| {
        route
            .get("/parts/*")
            .with_path_extractor::<PathExtractor>()
            .to(parts_handler);
    })
}

#[cfg(test)]
mod tests {
    use {super::*, gotham::test::TestServer, hyper::StatusCode, moxie_dom::embed::WebRuntime};

    #[test]
    fn extracts_one_component() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/parts/head")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = String::from_utf8(response.read_body().unwrap()).unwrap();
        assert_eq!(
            &body,
            "<ul>
  <li>head</li>
</ul>",
        );
    }

    #[test]
    fn extracts_multiple_components() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/parts/head/shoulders/knees/toes")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = String::from_utf8(response.read_body().unwrap()).unwrap();
        assert_eq!(
            &body,
            &"<ul>
  <li>head</li>
  <li>shoulders</li>
  <li>knees</li>
  <li>toes</li>
</ul>",
        );
    }

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
