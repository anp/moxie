use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Name(String);

impl From<String> for Name {
    fn from(s: String) -> Self {
        Name(s.to_string())
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0)
    }
}
