#![feature(proc_macro_diagnostic)]
#![deny(clippy::all)]

extern crate proc_macro;

mod component;
mod mox;
mod runtime;

use {proc_macro::TokenStream, syn::Ident};

fn component_storage_ty_name(component_name: &Ident) -> Ident {
    let mut comp_name_str = component_name.to_string();
    comp_name_str.push_str("Storage");
    Ident::new(&comp_name_str, component_name.span())
}

fn component_function_name(component_name: &Ident) -> Ident {
    let comp_name_str = component_name.to_string().to_lowercase();
    Ident::new(&comp_name_str, component_name.span())
}

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
    component::component_impl(attrs, input)
}
