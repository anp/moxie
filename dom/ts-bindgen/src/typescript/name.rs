use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use swc_ecma_ast::TsEntityName;

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Name {
    // TODO preserve span from TS file
    segments: Vec<String>,
    dotted: String,
}

impl Name {
    pub fn dot(&self, rhs: &Name) -> Self {
        let mut segments = self.segments.clone();
        segments.extend(rhs.segments.clone());
        Self::from(segments)
    }

    pub fn terminal(&self) -> &str {
        self.segments.last().unwrap()
    }
}

impl From<String> for Name {
    fn from(dotted: String) -> Self {
        let segments = dotted.split(".").map(ToString::to_string).collect();
        Self { segments, dotted }
    }
}

impl From<Vec<String>> for Name {
    fn from(segments: Vec<String>) -> Self {
        let dotted = segments.join(".");
        Self { segments, dotted }
    }
}

impl From<TsEntityName> for Name {
    fn from(entity: TsEntityName) -> Self {
        let mut segments = Vec::new();
        push_name_segments(entity, &mut segments);
        let dotted = segments.join(".");
        Self { segments, dotted }
    }
}

fn push_name_segments(name: TsEntityName, segments: &mut Vec<String>) {
    match name {
        TsEntityName::Ident(i) => segments.push(i.sym.to_string()),
        TsEntityName::TsQualifiedName(qualified) => {
            push_name_segments(qualified.left, segments);
            segments.push(qualified.right.sym.to_string());
        }
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.dotted
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.dotted)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.dotted)
    }
}
