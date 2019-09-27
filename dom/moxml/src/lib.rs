extern crate proc_macro;

use {
    proc_macro2::{Group, Ident, TokenStream, TokenTree},
    proc_macro_error::{filter_macro_errors, MacroError, ResultExt},
    quote::{quote, ToTokens},
    snax::{ParseError, SnaxAttribute, SnaxFragment, SnaxItem, SnaxSelfClosingTag, SnaxTag},
    std::convert::TryFrom,
};

#[proc_macro_hack::proc_macro_hack]
pub fn moxml(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    filter_macro_errors! {
        let item = snax::parse(input.into()).map_err(Error::SnaxError).unwrap_or_exit();
        let item = MoxItem::from(item);
        quote!(#item).into()
    }
}

enum MoxItem {
    Tag(MoxTag),
    Fragment(Vec<MoxItem>),
    Content(Content),
}

impl From<SnaxItem> for MoxItem {
    fn from(item: SnaxItem) -> Self {
        match item {
            SnaxItem::Tag(t) => MoxItem::Tag(MoxTag::from(t)),
            SnaxItem::SelfClosingTag(t) => MoxItem::Tag(MoxTag::from(t)),
            SnaxItem::Fragment(SnaxFragment { children }) => {
                MoxItem::Fragment(children.into_iter().map(MoxItem::from).collect())
            }
            SnaxItem::Content(atom) => MoxItem::Content(Content::from(atom)),
        }
    }
}

impl ToTokens for MoxItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MoxItem::Tag(tag) => tag.to_tokens(tokens),
            MoxItem::Fragment(children) => {
                for c in children {
                    c.to_tokens(tokens);
                }
            }
            MoxItem::Content(content) => content.to_tokens(tokens),
        }
    }
}

struct MoxTag {
    name: Ident,
    attributes: Vec<MoxAttr>,
    children: Vec<MoxItem>,
}

impl ToTokens for MoxTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.to_string();

        let mut attrs = quote!();
        self.attributes
            .iter()
            .map(ToTokens::to_token_stream)
            .for_each(|ts| attrs.extend(ts));

        let mut children = quote!();
        self.children
            .iter()
            .map(ToTokens::to_token_stream)
            .for_each(|ts| children.extend(ts));

        tokens.extend(quote!(
            // TODO this needs to be any topologically-aware function, not just an html element
            moxie_dom::element!(#name, |e| e
                #attrs
                .inner(|| {
                    #children
                })
            );
        ))
    }
}

impl From<SnaxTag> for MoxTag {
    fn from(
        SnaxTag {
            name,
            attributes,
            children,
        }: SnaxTag,
    ) -> Self {
        Self {
            name,
            attributes: attributes.into_iter().map(MoxAttr::from).collect(),
            children: children.into_iter().map(MoxItem::from).collect(),
        }
    }
}

impl From<SnaxSelfClosingTag> for MoxTag {
    fn from(SnaxSelfClosingTag { name, attributes }: SnaxSelfClosingTag) -> Self {
        Self {
            name,
            attributes: attributes.into_iter().map(MoxAttr::from).collect(),
            children: vec![],
        }
    }
}

enum MoxAttr {
    Simple { name: Ident, value: TokenTree },
    Handler { name: Ident, value: TokenTree },
}

impl ToTokens for MoxAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // unimplemented!("attr to tokens")
    }
}

impl From<SnaxAttribute> for MoxAttr {
    fn from(attr: SnaxAttribute) -> Self {
        match attr {
            SnaxAttribute::Simple { name, value } => MoxAttr::Simple { name, value },
        }
    }
}

enum Content {
    FormatExpr(TokenTree),
    RustExpr(TokenTree),
}

impl ToTokens for Content {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Content::FormatExpr(tt) | Content::RustExpr(tt) => tokens.extend(quote!(#tt)),
        }
    }
}

impl From<TokenTree> for Content {
    fn from(tt: TokenTree) -> Self {
        match tt {
            tt @ TokenTree::Ident(_) | tt @ TokenTree::Literal(_) | tt @ TokenTree::Punct(_) => {
                Content::RustExpr(tt)
            }
            TokenTree::Group(g) => {
                let mut tokens = g.stream().into_iter().peekable();
                if let Some(TokenTree::Punct(p)) = tokens.next() {
                    if p.as_char() == '%' {
                        let mut new_stream = quote!();
                        // TODO get all but the last element here too if its a %
                        new_stream.extend(tokens);
                        Content::FormatExpr(
                            Group::new(
                                g.delimiter(),
                                quote!(moxie_dom::text!(format!(#new_stream))),
                            )
                            .into(),
                        )
                    } else {
                        Content::RustExpr(g.into())
                    }
                } else {
                    Content::RustExpr(g.into())
                }
            }
        }
    }
}

enum Error {
    SnaxError(ParseError),
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
