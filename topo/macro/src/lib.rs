//! Procedural macro support crate for the `topo` crate.

#![deny(missing_docs)]

extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, AttrStyle, Attribute,
    AttributeArgs, Expr, ExprTuple, FnArg, ItemFn, Lit, Meta, NestedMeta, Pat, PatIdent, Signature,
    Token,
};

/// Transforms a function declaration into a topologically-nested function
/// which, when called, attaches its call subtopology to that of its caller's
/// (parent's).
///
/// TODO document slots
#[proc_macro_attribute]
pub fn nested(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: AttributeArgs = parse_macro_input!(args);
    let mut input_fn: ItemFn = syn::parse(input).unwrap();

    let inner_block = input_fn.block;
    input_fn.block = if let Some(slot_expr) = slot_expr(args, &mut input_fn.sig) {
        parse_quote! {{ topo::call_in_slot(#slot_expr, move || #inner_block) }}
    } else {
        parse_quote! {{ topo::call(move || #inner_block) }}
    };

    quote::quote_spanned!(input_fn.span()=>
        #[track_caller]
        #input_fn
    )
    .into()
}

/// collect all slot expressions into a single tuple to pass to topo
fn slot_expr(attr_args: AttributeArgs, sig: &mut Signature) -> Option<ExprTuple> {
    let mut elems: Punctuated<Expr, Token![,]> = slot_from_args(&attr_args).into_iter().collect();

    for input in &mut sig.inputs {
        if is_slot(input) {
            let slot = match input {
                FnArg::Receiver(_) => unimplemented!(),
                FnArg::Typed(t) => pat_to_expr(&*t.pat),
            };

            elems.push(slot);
        }
    }

    if !elems.is_empty() {
        Some(ExprTuple { attrs: vec![], paren_token: sig.paren_token, elems })
    } else {
        None
    }
}

/// parse the attribute arguments, retrieving an an expression to use as part of
/// the slot
fn slot_from_args(args: &[NestedMeta]) -> Option<Expr> {
    assert!(args.len() <= 1);

    args.get(0).map(|arg| match arg {
        NestedMeta::Meta(Meta::NameValue(kv)) => {
            assert!(
                kv.path.is_ident("slot"),
                "only `slot = \"...\" argument is supported by #[nested]"
            );

            match &kv.lit {
                Lit::Str(l) => l.parse().unwrap(),
                _ => panic!("`slot` argument accepts a string literal"),
            }
        }
        _ => panic!("only `slot = \"...\" argument is supported by #[nested]"),
    })
}

/// extract an expression from this pattern which references each binding
/// exposed
fn pat_to_expr(pat: &Pat) -> Expr {
    match pat {
        Pat::Ident(PatIdent { ident, .. }) => {
            let and = Token![&](ident.span());
            parse_quote!(#and #ident)
        }
        _ => unimplemented!("#[slot] only supported on simplest argument expressions"),
    }
}

/// ugly hack -- strips slot attribute when returning bool
fn is_slot(arg: &mut FnArg) -> bool {
    let attrs = match arg {
        FnArg::Receiver(r) => &mut r.attrs,
        FnArg::Typed(t) => &mut t.attrs,
    };

    let to_remove = attrs
        .iter()
        .enumerate()
        .find_map(|(i, attr)| if is_slot_attr(attr) { Some(i) } else { None });

    if let Some(idx) = to_remove {
        attrs.remove(idx);
        true
    } else {
        false
    }
}

fn is_slot_attr(attr: &Attribute) -> bool {
    if attr.style != AttrStyle::Outer || !attr.tokens.is_empty() {
        false
    } else {
        attr.path.is_ident("slot")
    }
}
