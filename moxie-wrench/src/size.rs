use {noisy_float::prelude::*, winit::dpi::LogicalSize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Size {
    pub width: R64,
    pub height: R64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width: r64(width),
            height: r64(height),
        }
    }
}

impl Into<LogicalSize> for Size {
    fn into(self) -> LogicalSize {
        LogicalSize::new(*self.width.as_ref(), *self.height.as_ref())
    }
}
