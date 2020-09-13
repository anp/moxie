use moxie_dom::{
    prelude::*,
    raw::{
        testing::{Query, TargetExt},
        Node,
    },
};
use std::ops::Deref;
use tracing::*;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
pub async fn add_2_todos() {
    let test = Test::new();
    let (first, second) = ("learn testing", "be cool");
    test.add_todo(first).await;
    test.add_todo(second).await;
    test.assert_todos(&[first, second]).await;
}

#[wasm_bindgen_test]
pub async fn add_default_todos() {
    Test::new().add_default_todos().await;
}

struct Test {
    root: Node,
}

impl Deref for Test {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

impl Test {
    fn new() -> Self {
        super::setup_tracing();
        let root = document().create_element("div");
        document().body().append_child(&root);
        super::boot(root.expect_concrete().clone());
        Test { root }
    }

    #[track_caller]
    async fn assert_todos(&self, expected: &[&str]) {
        assert_eq!(
            self.query_selector_all(".todo-list li")
                .iter()
                .map(|t| t.get_inner_text())
                .collect::<Vec<_>>(),
            expected
        );
    }

    async fn add_todo(&self, todo: &str) {
        self.input().keyboardln(todo);
        // wait for it to show up
        // TODO make sure it shows up at the end
        // TODO assert there's only one matching <li>
        self.find().by_text(todo).until().many().await.unwrap();
    }

    async fn add_default_todos(&self) {
        info!("adding default TODOs");
        let expected = &[TODO_ITEM_ONE, TODO_ITEM_TWO, TODO_ITEM_THREE];
        for todo in expected {
            self.add_todo(todo).await;
        }
        self.assert_todos(expected).await;
    }

    fn input(&self) -> Node {
        self.find().by_placeholder_text(INPUT_PLACEHOLDER).one().unwrap()
    }
}

impl Drop for Test {
    fn drop(&mut self) {
        document().body().remove_child(&self.root);
        // TODO blur active element just to be safe
        // TODO stop app and block until cleaned up
        // TODO clear local storage
    }
}

const INPUT_PLACEHOLDER: &str = "What needs to be done?";
const TODO_ITEM_ONE: &str = "buy some cheese";
const TODO_ITEM_TWO: &str = "feed the cat";
const TODO_ITEM_THREE: &str = "book a doctors appointment";
