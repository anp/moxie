//! Procedural macro support crate for the `illicit` crate.

#![deny(missing_docs)]

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::quote;
use syn::{
    parse::Parser, parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned,
    Attribute, FnArg, ItemFn, Local, PatType, Stmt, Token, Type, TypeReference,
};

/// Defines required [topo::Env] values for a function. Binds the provided types
/// as if references to them were implicit function arguments.
///
/// # Panics
///
/// Will cause the annotated function to panic if it is invoked without the
/// requested type in its `topo::Env`.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn from_env(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn: ItemFn = parse_macro_input!(input);

    let args = Punctuated::<FnArg, Token![,]>::parse_terminated.parse(args).unwrap();
    if args.is_empty() {
        abort_call_site!("must specify >=1 one argument");
    }

    let doc_prelude = "
# Environment Expectations

This function requires the following types to be visible to [`illicit::get`] and will
panic otherwise:
";

    for line in doc_prelude.lines() {
        input_fn.attrs.push(parse_quote!(#[doc = #line]));
    }

    for arg in args {
        let arg = match arg {
            FnArg::Receiver(rec) => abort!(rec.span(), "can't receive self by-environment"),
            FnArg::Typed(pt) => pt,
        };
        let (stmt, doc_attr) = bind_env_reference(&arg);
        input_fn.block.stmts.insert(0, Stmt::Local(stmt));
        input_fn.attrs.push(doc_attr);
    }

    quote::quote!(#input_fn).into()
}

/// Create a local expect assignment expression from the `pattern: &type`
/// pair which is passed.
fn bind_env_reference(arg: &PatType) -> (Local, Attribute) {
    let arg_span = arg.span();

    let (init_expr, ty_tokens) = match &*arg.ty {
        Type::Reference(TypeReference { lifetime, mutability, elem, .. }) => {
            if mutability.is_some() {
                abort!(mutability.span(), "mutable references cannot be passed by environment");
            }

            if lifetime.is_some() {
                abort!(
                    lifetime.span(),
                    "cannot bind to concrete lifetimes for environment references"
                );
            }

            (parse_quote!(&*illicit::expect::<#elem>()), quote!(#elem))
        }

        ty => (parse_quote!(illicit::expect::<#ty>().clone()), quote!(#ty)),
    };

    let ty_bullet = format!("* `{}`", ty_tokens);

    (
        Local {
            attrs: vec![],
            let_token: Token![let](arg_span),
            pat: *arg.pat.clone(),
            init: Some((Token![=](arg_span), Box::new(init_expr))),
            semi_token: Token![;](arg_span),
        },
        parse_quote!(#[doc = #ty_bullet]),
    )
}
