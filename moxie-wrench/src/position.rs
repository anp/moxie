use {noisy_float::prelude::*, winit::dpi::LogicalPosition};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
    pub x: R64,
    pub y: R64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: r64(x),
            y: r64(y),
        }
    }
}

impl From<LogicalPosition> for Position {
    fn from(logical: LogicalPosition) -> Self {
        Self::new(logical.x, logical.y)
    }
}

impl Into<LogicalPosition> for Position {
    fn into(self) -> LogicalPosition {
        LogicalPosition::new(*self.x.as_ref(), *self.y.as_ref())
    }
}
