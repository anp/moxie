//! These are compile time tests, but they shouldn't take any time to run, so we don't ignore them.
use mox::mox;

struct Tag();

impl Tag {
    fn single_dash(self, _value: &str) -> Self {
        self
    }

    fn build(self) {}
}

fn unused<T>(_x: T) {}

#[test]
pub fn single_dash() {
    let custom_name = || Tag();
    unused(|| mox!(<custom-name/>));
}

#[test]
pub fn single_dash_attr() {
    let tag = || Tag();
    unused(|| mox!(<tag single-dash="testing"/>));
}

#[test]
pub fn multi_dash() {
    let custom_multi_dash_name = || Tag();
    unused(|| mox!(<custom-multi-dash-name></custom-multi-dash-name>));
}

#[test]
pub fn trailing_dash() {
    let custom_trailing_dash_ = || Tag();
    unused(|| mox!(<custom-trailing-dash-></custom-trailing-dash->));
}

#[test]
pub fn keywords() {
    let custom_for_loop_in_self = || Tag();
    unused(|| mox!(<custom-for-loop-in-self></custom-for-loop-in-self>));
}

#[test]
pub fn leading_keyword() {
    let for_loop_in_self = || Tag();
    unused(|| mox!(<for-loop-in-self></for-loop-in-self>));
}
