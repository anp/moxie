//! Procedural macro support crate for the `topo` crate.

#![deny(missing_docs)]

extern crate proc_macro;
use {
    proc_macro::TokenStream,
    quote::ToTokens,
    syn::{
        parse::Parser, punctuated::Punctuated, spanned::Spanned, FnArg, Local, PatType, Stmt, Token,
    },
};

/// Transforms a function declaration into a topologically-nested function which, when called,
/// attaches its call subtopology to that of its caller's (parent's).
#[proc_macro_attribute]
pub fn nested(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn: syn::ItemFn = syn::parse_macro_input!(input);
    let inner_block = input_fn.block;

    input_fn.attrs.push(syn::parse_quote!(#[track_caller]));
    input_fn.block = syn::parse_quote! {{ topo::call(|| #inner_block) }};

    quote::quote!(#input_fn).into()
}
