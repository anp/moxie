use mox::mox;

#[derive(Debug, PartialEq)]
struct Tag {
    name: String,
    children: Vec<Tag>,
    optional: bool,
}

fn built() -> TagBuilder {
    TagBuilder::default()
}

#[derive(Default)]
struct TagBuilder {
    name: Option<String>,
    children: Vec<Tag>,
    optional: bool,
}

impl TagBuilder {
    fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    fn child(mut self, child: TagBuilder) -> Self {
        self.children.push(child.build());
        self
    }

    fn optional(mut self) -> Self {
      self.optional = true;
      self
    }

    fn build(self) -> Tag {
      Tag {
        name: self.name.unwrap(),
        children: self.children,
        optional: self.optional,
      }
    }
}

#[test]
fn method_syntax() {
    let expected = Tag {
        name: String::from("alice"),
        children: vec![
          Tag {
            name: String::from("bob"),
            children: vec![],
            optional: false,
          }
        ],
        optional: true,
    };

    assert_eq!(
        mox! {
            <built name="alice" {optional()}>
                <built {name("bob")}/>
            </built>
        },
        expected
    );
}
