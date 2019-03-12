#![feature(proc_macro_diagnostic)]
#![deny(clippy::all)]

extern crate proc_macro;

mod component;
mod mox;
mod runtime;

use {proc_macro::TokenStream, syn::parse_macro_input};

#[proc_macro]
pub fn mox(input: TokenStream) -> TokenStream {
    mox::mox_impl(input.into()).into()
}

#[proc_macro_attribute]
pub fn runtime(attrs: TokenStream, input: TokenStream) -> TokenStream {
    runtime::runtime_impl(attrs, input)
}

#[proc_macro_attribute]
pub fn component(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let makro: component::ComponentMacro = parse_macro_input!(input);
    let module = makro.expand();
    let tokens = quote::quote!(#module);
    tokens.into()
}
