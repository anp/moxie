//! These are compile time tests, but they shouldn't take any time to run, so we don't ignore them.
use mox::mox;
use moxie_dom::elements::html::div;

fn unused<T>(_x: T) {}

#[test]
pub fn single_word() {
    let custom_name = || div();
    unused(|| mox!(<custom-name/>));
}

#[test]
pub fn multi_word() {
    let custom_multi_word_name = || div();
    unused(|| mox!(<custom-multi-word-name></custom-multi-word-name>));
}

#[test]
pub fn trailing_dash() {
    let custom_trailing_dash_ = || div();
    unused(|| mox!(<custom-trailing-dash-></custom-trailing-dash->));
}

#[test]
pub fn keywords() {
    let custom_for_loop_in_self = || div();
    unused(|| mox!(<custom-for-loop-in-self></custom-for-loop-in-self>));
}

#[test]
pub fn leading_keyword() {
    let for_loop_in_self = || div();
    unused(|| mox!(<for-loop-in-self></for-loop-in-self>));
}
