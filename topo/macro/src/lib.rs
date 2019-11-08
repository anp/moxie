//! Procedural macro support crate for the `topo` crate.

#![deny(missing_docs)]

extern crate proc_macro;
use {proc_macro::TokenStream, syn::export::TokenStream2};

/// Transforms a function declaration into a topologically-nested function invoked with macro syntax
/// to attach its call tree's (sub)topology to the parent topology.
///
/// A macro transformation is used to capture unique callsite information from the invoking
/// function. In the current implementation, we synthesize a unique [`std::any::TypeId`] at each
/// callsite which can be used to identify the chain of topological invocations.
#[proc_macro_attribute]
pub fn nested(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    // let mut input_fn: syn::ItemFn = syn::parse_macro_input!(input);

    // TODO insert the Id entrypoint at the top of the function

    // quote::quote!(#input_fn).into()
    input
}
