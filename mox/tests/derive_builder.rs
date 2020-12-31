use derive_builder::Builder;
use mox::mox;
use std::error::Error;

#[derive(Builder, Debug, PartialEq)]
#[builder(pattern = "owned", setter(into))]
struct ToBuild {
    // TODO(colin-kiegel/rust-derive-builder#168) cleanup
    #[builder(setter(name = "name"))]
    name: String,
    #[builder(setter(name = "height"))]
    height: i32,
    #[builder(private, default)]
    children: Vec<ToBuild>,
}

impl ToBuild {
    fn into_child(self) -> Self {
        self
    }
}

impl ToBuildBuilder {
    fn child(mut self, child: ToBuild) -> Self {
        let children = if let Some(c) = self.children.as_mut() {
            c
        } else {
            self.children = Some(Vec::new());
            self.children.as_mut().unwrap()
        };

        children.push(child);
        self
    }
}

fn built() -> ToBuildBuilder {
    ToBuildBuilder::default()
}

#[test]
fn simple() -> Result<(), Box<dyn Error>> {
    let expected = ToBuild {
        name: String::from("alice"),
        height: 3,
        children: vec![ToBuild { name: String::from("bob"), height: 1, children: vec![] }],
    };

    assert_eq!(
        mox! {
            <built name="alice" height=3>
                // TODO clean up with fallible tags
                { mox!(<built name="bob" height=1 />)? }
            </built>
        }?,
        expected
    );
    Ok(())
}
