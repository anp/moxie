use {
    noisy_float::prelude::*,
    std::{
        fmt::{Debug, Formatter, Result as FmtResult},
        hash::{Hash, Hasher},
        ops::{Deref, DerefMut},
    },
    webrender::api::ColorF,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Color {
    r: R32,
    g: R32,
    b: R32,
    a: R32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r32(r),
            g: r32(g),
            b: r32(b),
            a: r32(a),
        }
    }
}

impl From<ColorF> for Color {
    fn from(c: ColorF) -> Color {
        Color {
            r: r32(c.r),
            g: r32(c.g),
            b: r32(c.b),
            a: r32(c.a),
        }
    }
}

impl Into<ColorF> for Color {
    fn into(self) -> ColorF {
        ColorF {
            r: *self.r.as_ref(),
            g: *self.g.as_ref(),
            b: *self.b.as_ref(),
            a: *self.a.as_ref(),
        }
    }
}
