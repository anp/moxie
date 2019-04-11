use {
    crate::color::Color,
    euclid::{TypedPoint2D, TypedRect, TypedSize2D},
    moxie::*,
    noisy_float::prelude::*,
    webrender::api::*,
};

#[props]
pub struct Rect {
    pub x: R32,
    pub y: R32,
    pub width: R32,
    pub height: R32,
    pub color: Color,
}

impl Component for Rect {
    fn compose(
        scp: Scope,
        Self {
            x,
            y,
            width,
            height,
            color,
        }: Self,
    ) {
        let color: ColorF = color.into();
        scp.record((
            SpecificDisplayItem::Rectangle(RectangleDisplayItem { color }),
            LayoutPrimitiveInfo::new(TypedRect::new(
                TypedPoint2D::new(x.raw() as f32, y.raw() as f32),
                TypedSize2D::new(width.raw() as f32, height.raw() as f32),
            )),
        ));
    }
}
