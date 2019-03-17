#![feature(proc_macro_diagnostic)]
#![deny(clippy::all)]

extern crate proc_macro;

mod mox;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn props(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut derives: TokenStream =
        quote::quote!(#[derive(Clone, Debug, Eq, Hash, PartialEq)]).into();
    derives.extend(input);
    derives
}

#[proc_macro]
pub fn mox(input: TokenStream) -> TokenStream {
    mox::mox_impl(input.into()).into()
}
