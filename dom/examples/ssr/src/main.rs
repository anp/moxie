#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate serde_derive;

use augdom::Dom;
use gotham::{
    router::{builder::*, Router},
    state::{FromState, State},
};
use mox::mox;
use moxie_dom::{
    elements::text_content::{li, ul, Ul},
    embed::DomLoop,
    prelude::*,
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
fn simple_list(items: &[String]) -> Ul {
    let mut list = ul();
    for item in items {
        list = list.child(mox!(<li>{% "{}", item }</li>));
    }
    list.build()
}

fn parts_handler(state: State) -> (State, String) {
    let parts = {
        let path = PathExtractor::borrow_from(&state);
        path.parts.to_owned()
    };
    let web_div = augdom::create_virtual_element("div");
    let mut renderer = DomLoop::new_virtual(web_div.clone(), move || simple_list(&parts));
    renderer.run_once();
    (state, web_div.pretty_outer_html(2))
}

fn router() -> Router {
    build_simple_router(|route| {
        route.get("/parts/*").with_path_extractor::<PathExtractor>().to(parts_handler);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn extracts_one_component() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server.client().get("http://localhost/parts/head").perform().unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = String::from_utf8(response.read_body().unwrap()).unwrap();
        assert_eq!(
            &body,
            r#"<div>
  <ul>
    <li>head</li>
  </ul>
</div>"#,
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
            &r#"<div>
  <ul>
    <li>head</li>
    <li>shoulders</li>
    <li>knees</li>
    <li>toes</li>
  </ul>
</div>"#,
        );
    }

    #[test]
    fn basic_list_prerender() {
        let root = augdom::create_virtual_element("div");
        let mut tester = DomLoop::new_virtual(root.clone(), move || {
            mox! {
                <ul class="listywisty">
                    <li>"first"</li>
                    <li class="item">"second"</li>
                    <li>"third"</li>
                </ul>
            }
        });

        tester.run_once();

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
