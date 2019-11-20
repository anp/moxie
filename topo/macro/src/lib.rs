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
    let mut input_fn: syn::ItemFn = syn::parse_macro_input!(input);

    let tmp = std::mem::replace(&mut input_fn.attrs, Vec::new());
    let mut doc_attrs = Vec::new();

    for attr in tmp {
        match attr.parse_meta() {
            Ok(syn::Meta::NameValue(syn::MetaNameValue { ref path, .. })) => {
                if let Some(id) = path.get_ident() {
                    if *id == "doc" {
                        doc_attrs.push(attr);
                    }
                }
            }
            _ => input_fn.attrs.push(attr),
        }
    }

    let docs_fn_sig = docs_fn_signature(&input_fn);

    let mangled_name = syn::Ident::new(
        &format!("__{}_impl", &input_fn.sig.ident),
        input_fn.sig.ident.span(),
    );

    let macro_name = std::mem::replace(&mut input_fn.sig.ident, mangled_name.clone());

    quote::quote!(
        topo::unstable_make_topo_macro!(
            #macro_name #mangled_name
            match ($($arg:expr),*)
            subst ($($arg),*)
            doc (
                #(#doc_attrs)*
                #docs_fn_sig
            )
        );

        #[doc(hidden)]
        #input_fn
    )
    .into()
}

/// Creates a series of `#[doc = "..."]` attributes for the input function's signature. This is
/// useful because macros can't have typed return values, arguments, or generics! We embed this
/// output into the final macro output so that it the rendered docs display the type signatures.
fn docs_fn_signature(input_fn: &syn::ItemFn) -> TokenStream2 {
    let doc_fn_sig = syn::ItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        sig: input_fn.sig.clone(),
        block: Box::new(syn::Block {
            brace_token: syn::token::Brace {
                span: proc_macro::Span::call_site().into(),
            },
            stmts: vec![],
        }),
    };

    let doc_fn_sig = quote::quote!(#doc_fn_sig).to_string();

    use std::io::prelude::*;
    let doc_fn_sig = if let Ok(mut rustfmt) = std::process::Command::new("rustfmt")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        let stdin = rustfmt.stdin.as_mut().unwrap();
        stdin.write_all(doc_fn_sig.as_bytes()).unwrap();

        let output = rustfmt.wait_with_output().unwrap();

        if output.status.success() {
            String::from_utf8(output.stdout).unwrap()
        } else {
            doc_fn_sig
        }
    } else {
        doc_fn_sig
    };

    ["", "# Signature", "", "```text", &doc_fn_sig, "```"]
        .iter()
        .map(|&l| quote::quote!(#[doc = #l]))
        .fold(quote::quote!(), |a, b| quote::quote!(#a #b))
}
