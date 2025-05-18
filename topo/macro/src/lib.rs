//! Procedural macro support crate for the `topo` crate.

use proc_macro::TokenStream;
use syn::{parse_macro_input, parse_quote, spanned::Spanned, Expr, ItemFn};

/// FIXME add docs
#[proc_macro_attribute]
pub fn nested(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut slot: Option<Expr> = None;
    let slot_parser = syn::meta::parser(|meta| {
        if !meta.path.is_ident("slot") {
            return Err(meta.error("only `slot` argument is supported"));
        }
        if slot.is_some() {
            return Err(meta.error("only one `slot` argument is supported"));
        }
        slot = Some(meta.value()?.parse()?);
        Ok(())
    });

    parse_macro_input!(args with slot_parser);
    match nested_inner(slot, input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn nested_inner(slot: Option<Expr>, input: TokenStream) -> syn::Result<TokenStream> {
    let mut input_fn: ItemFn = syn::parse(input)?;

    let inner_block = input_fn.block;
    input_fn.block = if let Some(slot) = slot {
        parse_quote! {{
            topo::call_in_slot(#slot, move || #inner_block)
        }}
    } else {
        parse_quote! {{ topo::call(move || #inner_block) }}
    };

    Ok(quote::quote_spanned!(input_fn.span()=>
        #[track_caller]
        #input_fn
    )
    .into())
}
