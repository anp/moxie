//! Procedural macro support crate for the `topo` crate.

use proc_macro::TokenStream;
use syn::{parse_macro_input, parse_quote, spanned::Spanned, Ident, ItemFn};

/// FIXME add docs
#[proc_macro_attribute]
pub fn nested(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut slot: Option<Ident> = None;
    let slot_parser = syn::meta::parser(|meta| {
        if !meta.path.is_ident("slot") {
            return Err(meta.error("only `slot` argument is supported"));
        }
        if slot.is_some() {
            return Err(meta.error("only one `slot` argument is supported"));
        }
        slot = Some(ident_from_ident_or_string(meta.value()?)?);
        Ok(())
    });

    parse_macro_input!(args with slot_parser);
    match nested_inner(slot, input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn nested_inner(slot: Option<Ident>, input: TokenStream) -> syn::Result<TokenStream> {
    let mut input_fn: ItemFn = syn::parse(input)?;

    let inner_block = input_fn.block;
    input_fn.block = if let Some(slot_expr) = slot {
        parse_quote! {{
            topo::call_in_slot(#slot_expr, move || #inner_block)
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

fn ident_from_ident_or_string(input: &syn::parse::ParseBuffer<'_>) -> syn::Result<Ident> {
    if let Ok(ident) = input.parse::<Ident>() {
        return Ok(ident);
    }
    let string: syn::LitStr = input.parse()?;
    let ident = Ident::new(&string.value(), string.span());
    Ok(ident)
}
