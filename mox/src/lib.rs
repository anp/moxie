#![feature(proc_macro_diagnostic)]
#![deny(clippy::all)]

extern crate proc_macro;

mod mox;

use proc_macro::TokenStream;

#[proc_macro]
pub fn mox(input: TokenStream) -> TokenStream {
    mox::mox_impl(input.into()).into()
}
