use mox::mox;

#[derive(Debug, PartialEq)]
struct Tag {
    name: String,
    children: Vec<Tag>,
}

fn built() -> TagBuilder {
    TagBuilder::default()
}

#[derive(Default)]
struct TagBuilder {
    name: Option<String>,
    children: Vec<Tag>,
}

impl TagBuilder {
    fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    fn child(mut self, child: Tag) -> Self {
        self.children.push(child);
        self
    }

    fn build(self) -> Tag {
        Tag { name: self.name.unwrap(), children: self.children }
    }
}

#[test]
fn simple() {
    let expected = Tag {
        name: String::from("alice"),
        children: vec![Tag { name: String::from("bob"), children: vec![] }],
    };

    assert_eq!(
        mox! {
            <built name="alice">
                <built name="bob"/>
            </built>
        },
        expected
    );
}
