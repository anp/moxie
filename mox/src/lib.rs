#![feature(proc_macro_diagnostic)]
#![deny(clippy::all)]

extern crate proc_macro;

mod component;
mod mox;
mod runtime;

use {
    proc_macro::TokenStream,
    syn::{parse_macro_input, spanned::Spanned},
};

#[proc_macro]
pub fn mox(input: TokenStream) -> TokenStream {
    mox::mox_impl(input.into()).into()
}

#[proc_macro_attribute]
pub fn runtime(attrs: TokenStream, input: TokenStream) -> TokenStream {
    runtime::runtime_impl(attrs, input)
}

#[proc_macro_attribute]
pub fn component(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs2: proc_macro2::TokenStream = attrs.clone().into();
    let attrs_span = attrs2.span();
    let comp_fn: syn::ItemFn = parse_macro_input!(input);
    let makro = component::ComponentMacro::new(attrs_span, comp_fn).unwrap();
    let items = makro.expand();
    let tokens = quote::quote!(#(#items)*);
    tokens.into()
}
