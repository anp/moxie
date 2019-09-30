extern crate proc_macro;

use {
    proc_macro2::{Delimiter, Group, Ident, TokenStream, TokenTree},
    proc_macro_error::{call_site_error, filter_macro_errors, span_error, MacroError, ResultExt},
    quote::{quote, ToTokens},
    snax::{ParseError, SnaxAttribute, SnaxFragment, SnaxItem, SnaxSelfClosingTag, SnaxTag},
};

#[proc_macro_hack::proc_macro_hack]
pub fn mox(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    filter_macro_errors! {
        let item = snax::parse(input.into()).map_err(Error::SnaxError).unwrap_or_exit();
        let item = MoxItem::from(item);
        quote!(#item).into()
    }
}

enum MoxItem {
    Tag(MoxTag),
    TagNoChildren(MoxTagNoChildren),
    Fragment(Vec<MoxItem>),
    Content(TokenTree),
}

impl From<SnaxItem> for MoxItem {
    fn from(item: SnaxItem) -> Self {
        match item {
            SnaxItem::Tag(t) => MoxItem::Tag(MoxTag::from(t)),
            SnaxItem::SelfClosingTag(t) => MoxItem::TagNoChildren(MoxTagNoChildren::from(t)),
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
                    new_stream = quote!(text!(format!(#new_stream)));
                }
            }
        }
        tt @ TokenTree::Ident(_) | tt @ TokenTree::Literal(_) => {
            new_stream = quote!(text!(#tt));
        }
        TokenTree::Punct(p) => span_error!(p.span(), "'{}' not valid in item position", p),
    }
    Group::new(Delimiter::Brace, new_stream).into()
}

impl ToTokens for MoxItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MoxItem::Tag(tag) => tag.to_tokens(tokens),
            MoxItem::TagNoChildren(tag) => tag.to_tokens(tokens),
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
    fn_args: Option<MoxArgs>,
    attributes: Vec<MoxAttr>,
    children: Vec<MoxItem>,
}

impl ToTokens for MoxTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tag_to_tokens(
            &self.name,
            &self.fn_args,
            &self.attributes,
            Some(&self.children),
            tokens,
        );
    }
}

fn args_and_attrs(snaxttrs: Vec<SnaxAttribute>) -> (Option<MoxArgs>, Vec<MoxAttr>) {
    let mut snaxs = snaxttrs.into_iter().peekable();
    let mut args = None;

    if let Some(first) = snaxs.peek() {
        let name = match first {
            SnaxAttribute::Simple { name, .. } => name,
        };

        if name == "_" {
            let first = snaxs.next().unwrap();
            args = Some(MoxArgs {
                value: match first {
                    SnaxAttribute::Simple { value, .. } => value,
                },
            });
        }
    }

    (args, snaxs.map(MoxAttr::from).collect())
}

fn tag_to_tokens(
    name: &Ident,
    fn_args: &Option<MoxArgs>,
    attributes: &[MoxAttr],
    children: Option<&[MoxItem]>,
    stream: &mut TokenStream,
) {
    // this needs to be nested within other token groups, must be accumulated separately from stream
    let mut contents = quote!();

    attributes
        .iter()
        .map(ToTokens::to_token_stream)
        .for_each(|ts| contents.extend(ts));

    if let Some(items) = children {
        let mut children = quote!();
        items
            .iter()
            .map(ToTokens::to_token_stream)
            .for_each(|ts| children.extend(quote!(#ts;)));

        contents.extend(quote!(
            .inner(|| {
                #children
            })
        ));
    } else if !contents.is_empty() {
        // if there were attributes or handlers installed but there isn't an inner function to call
        // with its own return type, the previous calls probably return references that can't return
        contents.extend(quote!(;));
    }

    let fn_args = fn_args.as_ref().map(|args| match &args.value {
        TokenTree::Group(g) => {
            // strip trailing commas that would bork macro parsing
            let mut tokens: Vec<TokenTree> = g.stream().into_iter().collect();

            let last = tokens
                .get(tokens.len() - 1)
                .expect("function argument delimiters must contain some tokens");

            if last.to_string() == "," {
                tokens.truncate(tokens.len() - 1);
            }

            let mut without_delim = TokenStream::new();
            without_delim.extend(tokens);
            quote!(#without_delim)
        }
        _ => unimplemented!("bare function args (without a paired delimiter) aren't supported yet"),
    });

    let invocation = if contents.is_empty() {
        quote!(#name!(#fn_args))
    } else {
        if fn_args.is_some() {
            unimplemented!(
                "can't emit function arguments at the same time as attributes or children yet"
            )
        }
        quote!(#name!(|_e| { _e #contents }))
    };

    stream.extend(invocation);
}

impl From<SnaxTag> for MoxTag {
    fn from(
        SnaxTag {
            name,
            attributes,
            children,
        }: SnaxTag,
    ) -> Self {
        let (fn_args, attributes) = args_and_attrs(attributes);
        Self {
            name,
            fn_args,
            attributes,
            children: children.into_iter().map(MoxItem::from).collect(),
        }
    }
}

struct MoxTagNoChildren {
    name: Ident,
    fn_args: Option<MoxArgs>,
    attributes: Vec<MoxAttr>,
}

impl ToTokens for MoxTagNoChildren {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tag_to_tokens(&self.name, &self.fn_args, &self.attributes, None, tokens);
    }
}

impl From<SnaxSelfClosingTag> for MoxTagNoChildren {
    fn from(SnaxSelfClosingTag { name, attributes }: SnaxSelfClosingTag) -> Self {
        let (fn_args, attributes) = args_and_attrs(attributes);
        Self {
            name,
            fn_args,
            attributes,
        }
    }
}

struct MoxArgs {
    value: TokenTree,
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
                let name_str = name.to_string();
                if name_str == "_" {
                    span_error!(
                        name.span(),
                        "anonymous attributes are only allowed in the first position"
                    );
                } else if name_str == "on" {
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
