extern crate proc_macro_hack;

use generational_arena::{Arena, Index};
use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use mox_macro::mox;

pub type Children = Vec<Node>;

pub struct View {
    id: Option<String>,
}

impl Element for View {
    fn create(&self, children: Children) -> Node {
        let attrs = if let Some(id) = self.id.as_ref() {
            vec![("id", id.to_owned())]
        } else {
            vec![]
        };

        Node::Elem {
            name: "div",
            attrs,
            children,
            events: (),
        }
    }
}

pub struct Link {
    pub href: String,
}

impl Element for Link {
    fn create(&self, children: Children) -> Node {
        Node::Elem {
            name: "a",
            attrs: vec![("href", self.href.clone())],
            events: (),
            children,
        }
    }
}

pub struct Text;

impl Element for Text {
    fn create(&self, children: Children) -> Node {
        Node::Elem {
            name: "span",
            attrs: vec![],
            events: (),
            children,
        }
    }
}

pub struct Input {
    pub value: String,
    pub _type: String,
}

impl Element for Input {
    fn create(&self, children: Children) -> Node {
        Node::Elem {
            name: "input",
            attrs: vec![
                ("value", self.value.to_owned()),
                ("type", self._type.to_owned()),
            ],
            children,
            events: (),
        }
    }
}

pub struct TextNode(pub String);

impl Element for TextNode {
    fn create(&self, children: Children) -> Node {
        assert_eq!(children.len(), 0);
        Node::Text(self.0.clone())
    }
}

#[derive(Debug)]
pub enum Node {
    Elem {
        name: &'static str,
        attrs: Vec<(&'static str, String)>,
        events: (),
        children: Vec<Node>,
    },
    Text(String),
}

// IDEA: if an element's value is a literal, try to parse it in proc macro?

pub trait Element {
    fn create(&self, children: Children) -> Node;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        // let macrod = mox! {
        //     <View id="container">
        //         <Input value="foo" _type="text"/>
        //         <Link href="/bar"/>
        //         <Text>hello world now</Text>
        //     </View>
        // };

        let manual = View {
            id: "container".to_owned().into(),
        }
        .create(vec![
            Input {
                value: "foo".into(),
                _type: "text".into(),
            }
            .create(vec![]),
            Link {
                href: "/bar".into(),
            }
            .create(vec![]),
            Text.create(vec![TextNode("hello world now".into()).create(vec![])]),
        ]);

        // assert_eq!(macrod, manual);
    }
}
