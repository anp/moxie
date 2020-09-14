//! Integration tests for TodoMVC.
//!
//! A module within the application rather than a "proper" integration test
//! because cargo and wasm-pack are conspiring to make that not build somehow.
//! The workaround for now is to make this a module of the app itself, so we
//! have to be on our best behavior and only use public API.

use moxie_dom::{
    prelude::*,
    raw::{
        testing::{Query, TargetExt},
        Node,
    },
};
use std::{fmt::Debug, ops::Deref};
use tracing::*;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
pub async fn add_2_todos() {
    let test = Test::new();
    test.add_todo("learn testing").await;
    test.add_todo("be cool").await;
}

#[wasm_bindgen_test]
pub async fn add_default_todos() {
    Test::new().add_default_todos().await;
}

#[allow(unused)] // TODO add back wasm_bindgen_test when we have a full page load to test autofocus
async fn initial_open_focuses_input() {
    let _test = Test::new();
    let focused = document().active_element().unwrap();
    assert_eq!(focused.get_attribute("className").unwrap(), "new-todo");
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
        // Please only use public functions from the crate, see module docs for
        // explanation.
        use super::{boot, setup_tracing};

        setup_tracing();
        let root = document().create_element("div");
        document().body().append_child(&root);
        boot(root.expect_concrete().clone());
        Test { root }
    }

    fn todos(&self) -> Vec<String> {
        self.query_selector_all(".todo-list li")
            .iter()
            .map(|t| t.get_inner_text())
            .collect::<Vec<_>>()
    }

    #[track_caller]
    fn assert_todos<Expected>(&self, expected: &[Expected])
    where
        String: PartialEq<Expected>,
        Expected: Debug,
    {
        assert_eq!(self.todos(), expected);
    }

    /// Add a new todo to the list, asserting that the list only grows by the
    /// one item.
    async fn add_todo(&self, todo: &str) {
        let mut expected = self.todos();
        expected.push(todo.to_owned());

        // actually input the new todo
        self.input().keyboardln(todo);
        // wait for it to show up
        self.find().by_text(todo).until().many().await.unwrap();
        self.assert_todos(&expected[..]);
    }

    async fn add_default_todos(&self) {
        info!("adding default TODOs");
        let expected = &[TODO_ITEM_ONE, TODO_ITEM_TWO, TODO_ITEM_THREE];
        for todo in expected {
            self.add_todo(todo).await;
        }
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
