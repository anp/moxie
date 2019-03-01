use {
    std::{
        fmt::{Debug, Formatter, Result as FmtResult},
        hash::{Hash, Hasher},
        ops::{Deref, DerefMut},
    },
    webrender::api::ColorF,
};

#[derive(Clone, Copy)]
pub struct Color(pub ColorF);

impl From<ColorF> for Color {
    fn from(c: ColorF) -> Color {
        Color(c)
    }
}

impl Hash for Color {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        macro_rules! h {
            ($f:ident) => {
                let scaled = (self.0).$f * 1000.0;
                (scaled as u64).hash(hasher);
            };
        }

        h!(r);
        h!(g);
        h!(b);
        h!(a);
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        macro_rules! c {
            ($f:ident) => {
                (self.0).$f == (other.0).$f
            };
        }

        c!(r) && c!(g) && c!(b) && c!(a)
    }
}
impl Eq for Color {}

impl Debug for Color {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Color")
            .field("r", &self.0.r)
            .field("g", &self.0.g)
            .field("b", &self.0.b)
            .field("a", &self.0.a)
            .finish()
    }
}

impl Deref for Color {
    type Target = ColorF;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
