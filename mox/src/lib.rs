#![feature(proc_macro_diagnostic)]
#![deny(clippy::all)]

extern crate proc_macro;

mod mox;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn props(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let ast: syn::DeriveInput = syn::parse2(input.clone()).unwrap();
    let name = &ast.ident;
    let ty_params = ast.generics.type_params().map(|p| &p.ident);
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut expanded = quote::quote!(#[derive(Clone, Debug, Eq, Hash, PartialEq)]);

    expanded.extend(input);

    expanded.extend(quote::quote! (
        impl #impl_generics ::moxie::typename::TypeName for #name #ty_generics #where_clause {
            fn fmt(f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let _ty_name = concat!(module_path!(), "::", stringify!(#name));
                ::moxie::typename::fmt::TypeFormatter::new(f, _ty_name)
                    #(
                        .type_param::< #ty_params >()
                    )*
                    .finish()
            }
        }
    ));

    expanded.into()
}

#[proc_macro]
pub fn mox(input: TokenStream) -> TokenStream {
    mox::mox_impl(input.into()).into()
}
