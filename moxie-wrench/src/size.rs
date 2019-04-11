use {noisy_float::prelude::*, winit::dpi::LogicalSize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Size {
    pub width: R32,
    pub height: R32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: r32(width),
            height: r32(height),
        }
    }
}

impl Into<LogicalSize> for Size {
    fn into(self) -> LogicalSize {
        LogicalSize::new(self.width.raw() as f64, self.height.raw() as f64)
    }
}
