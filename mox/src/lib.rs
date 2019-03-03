#![deny(clippy::all)]

extern crate proc_macro;

mod component;
mod mox;

use proc_macro::TokenStream;

#[proc_macro]
pub fn mox(input: TokenStream) -> TokenStream {
    mox::mox_impl(input.into()).into()
}

#[proc_macro_attribute]
pub fn component(attrs: TokenStream, input: TokenStream) -> TokenStream {
    component::component_impl(attrs.into(), input.into()).into()
}
