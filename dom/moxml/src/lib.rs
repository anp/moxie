extern crate proc_macro;

use {
    proc_macro2::{Delimiter, Group, Ident, TokenStream, TokenTree},
    proc_macro_error::{call_site_error, filter_macro_errors, span_error, MacroError, ResultExt},
    quote::{quote, ToTokens},
    snax::{ParseError, SnaxAttribute, SnaxFragment, SnaxItem, SnaxSelfClosingTag, SnaxTag},
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
    Content(TokenTree),
}

impl From<SnaxItem> for MoxItem {
    fn from(item: SnaxItem) -> Self {
        match item {
            SnaxItem::Tag(t) => MoxItem::Tag(MoxTag::from(t)),
            SnaxItem::SelfClosingTag(t) => MoxItem::Tag(MoxTag::from(t)),
            SnaxItem::Fragment(SnaxFragment { children }) => {
                MoxItem::Fragment(children.into_iter().map(MoxItem::from).collect())
            }
            SnaxItem::Content(atom) => MoxItem::Content(wrap_content_tokens(atom)),
        }
    }
}

fn wrap_content_tokens(tt: TokenTree) -> TokenTree {
    let mut new_stream = quote!(#tt);
    match tt {
        TokenTree::Group(g) => {
            let mut tokens = g.stream().into_iter();
            if let Some(TokenTree::Punct(p)) = tokens.next() {
                if p.as_char() == '%' {
                    // strip the percent sign off the front
                    new_stream = TokenStream::new();
                    new_stream.extend(tokens);

                    // TODO get all but the last element here too if its a %
                    new_stream = quote!(moxie_dom::text!(format!(#new_stream)));
                }
            }
        }
        tt @ TokenTree::Ident(_) | tt @ TokenTree::Literal(_) => {
            new_stream = quote!(moxie_dom::text!(#tt));
        }
        TokenTree::Punct(p) => span_error!(p.span(), "'{}' not valid in item position", p),
    }
    Group::new(Delimiter::Brace, new_stream).into()
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
    Handler { value: TokenTree },
}

impl ToTokens for MoxAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = match self {
            MoxAttr::Simple { name, value } => {
                let name = name.to_string();
                quote!(.attr(#name, #value))
            }
            MoxAttr::Handler {
                value: TokenTree::Group(g),
            } => {
                // remove the braces from the event handler args, these need to "splat" into a call
                let value = g.stream();
                quote!(.on(#value))
            }
            _ => call_site_error!("event handlers must be surrounded in braces"),
        };

        tokens.extend(stream);
    }
}

impl From<SnaxAttribute> for MoxAttr {
    fn from(attr: SnaxAttribute) -> Self {
        match attr {
            SnaxAttribute::Simple { name, value } => {
                if name.to_string() == "on" {
                    MoxAttr::Handler { value }
                } else {
                    MoxAttr::Simple { name, value }
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
