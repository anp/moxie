use crate::*;

mod button;

#[doc(inline)]
pub use button::*;

#[macro_export]
macro_rules! text {
    ($arg:expr) => {
        $crate::elements::Text($arg.into())
    };
    ($($arg:tt)+) => {
        $crate::elements::Text(format!( $($arg)* ))
    };
}

#[derive(Debug, PartialEq)]
pub struct Text(pub String);

impl Component for Text {
    fn contents(self) {
        let text_node = document().create_text_node(&self.0);
        produce_dom!(text_node, || {});
    }
}
