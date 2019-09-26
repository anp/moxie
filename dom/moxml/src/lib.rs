extern crate proc_macro;

use {
    proc_macro2::{Ident, TokenStream, TokenTree},
    proc_macro_error::{filter_macro_errors, MacroError, ResultExt},
    quote::{quote, ToTokens},
    snax::{ParseError, SnaxAttribute, SnaxFragment, SnaxItem, SnaxSelfClosingTag, SnaxTag},
    syn::parse::{Parse, ParseStream, Result as ParseResult},
};

#[proc_macro_hack::proc_macro_hack]
pub fn moxml(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    filter_macro_errors! {
        let item = snax::parse(input.into()).map_err(Error::SnaxError).unwrap_or_exit();
        expand_item(item).unwrap_or_exit().into()
    }
}

fn expand_item(item: SnaxItem) -> Result<TokenStream, Error> {
    let output = match item {
        SnaxItem::Tag(tag) => expand_tag(tag),
        SnaxItem::SelfClosingTag(tag) => expand_self_closing_tag(tag),
        SnaxItem::Fragment(SnaxFragment { children }) => expand_children(children),
        SnaxItem::Content(atom) => expand_content(atom),
    }?;
    Ok(output.into())
}

fn expand_element(name: Ident) -> TokenStream {
    // TODO call a custom component function instead of making an element with a name
    quote!(moxie_dom::element!(stringify!(#name)))
}

fn expand_attribute(attr: SnaxAttribute) -> Result<TokenStream, Error> {
    // TODO call .attr unless the name starts with `on`, then do event dispatch
    Ok(quote!())
}

fn expand_content(content: TokenTree) -> Result<TokenStream, Error> {
    Ok(match content {
        TokenTree::Ident(i) => quote!(#i),
        TokenTree::Punct(p) => quote!(#p),
        TokenTree::Literal(l) => quote!(#l),
        TokenTree::Group(g) => {
            let braced: BracedExpr = syn::parse2(g.stream()).map_err(Error::SynError)?;
            quote!(#braced)
        }
    })
}

enum BracedExpr {
    Format(TokenTree),
    Expr(TokenTree),
}

impl Parse for BracedExpr {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        unimplemented!("parsing")
    }
}

impl ToTokens for BracedExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        unimplemented!("expansion")
    }
}

fn expand_tag(
    SnaxTag {
        name,
        attributes,
        children,
    }: SnaxTag,
) -> Result<TokenStream, Error> {
    Ok(cons(
        cons(expand_element(name), expand_attributes(attributes)?),
        expand_children(children)?,
    ))
}

fn expand_self_closing_tag(
    SnaxSelfClosingTag { name, attributes }: SnaxSelfClosingTag,
) -> Result<TokenStream, Error> {
    Ok(cons(expand_element(name), expand_attributes(attributes)?))
}

fn expand_attributes(attrs: Vec<SnaxAttribute>) -> Result<TokenStream, Error> {
    cons_fallible_iter(attrs, expand_attribute)
}

fn expand_children(children: Vec<SnaxItem>) -> Result<TokenStream, Error> {
    cons_fallible_iter(children, expand_item)
}

fn cons_fallible_iter<T>(
    i: impl IntoIterator<Item = T>,
    op: impl FnMut(T) -> Result<TokenStream, Error>,
) -> Result<TokenStream, Error> {
    Ok(i.into_iter()
        .map(op)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .fold(quote!(), cons))
}

fn cons(mut l: TokenStream, r: TokenStream) -> TokenStream {
    l.extend(r);
    l
}

enum Error {
    SnaxError(ParseError),
    SynError(syn::Error),
}

impl Into<MacroError> for Error {
    fn into(self) -> MacroError {
        match self {
            Error::SnaxError(ParseError::UnexpectedEnd) => {
                MacroError::call_site(format!("input ends before expected"))
            }
            Error::SnaxError(ParseError::UnexpectedItem(item)) => {
                // TODO https://github.com/LPGhatguy/snax/issues/9
                MacroError::call_site(format!("did not expect {:?}", item))
            }
            Error::SnaxError(ParseError::UnexpectedToken(token)) => {
                MacroError::new(token.span(), format!("did not expect '{}'", token))
            }
            Error::SynError(e) => MacroError::new(e.span(), format!("{}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
