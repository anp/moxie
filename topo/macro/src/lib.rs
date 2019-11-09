//! Procedural macro support crate for the `topo` crate.

#![deny(missing_docs)]

extern crate proc_macro;
use {proc_macro::TokenStream, syn::export::TokenStream2};

/// Transforms a function declaration into a topologically-nested function which, when called,
/// attaches its call subtopology to that of its caller's (parent's).
#[proc_macro_attribute]
pub fn nested(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    // let mut input_fn: syn::ItemFn = syn::parse_macro_input!(input);

    // TODO insert the Id entrypoint at the top of the function

    // quote::quote!(#input_fn).into()
    input
}
