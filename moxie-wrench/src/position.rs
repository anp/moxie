use {noisy_float::prelude::*, winit::dpi::LogicalPosition};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
    pub x: R32,
    pub y: R32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: r32(x),
            y: r32(y),
        }
    }
}

impl From<LogicalPosition> for Position {
    fn from(logical: LogicalPosition) -> Self {
        Self::new(logical.x as f32, logical.y as f32)
    }
}

impl Into<LogicalPosition> for Position {
    fn into(self) -> LogicalPosition {
        LogicalPosition::new(self.x.raw() as f64, self.y.raw() as f64)
    }
}
