extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use proc_macro_error::{abort, emit_error, proc_macro_error, Diagnostic, Level, ResultExt};
use proc_macro_hack::proc_macro_hack;
use quote::{quote, ToTokens};
use snax::{ParseError, SnaxAttribute, SnaxFragment, SnaxItem, SnaxSelfClosingTag, SnaxTag};
use std::borrow::Cow;

#[proc_macro_error(allow_not_macro)]
#[proc_macro_hack]
pub fn mox(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = snax::parse(input.into()).map_err(Error::SnaxError).unwrap_or_abort();
    let item = MoxItem::from(item);
    quote!(#item).into()
}

enum MoxItem {
    Tag(MoxTag),
    TagNoChildren(MoxTagNoChildren),
    Fragment(Vec<MoxItem>),
    Content { span: Span, stream: TokenStream },
}

impl MoxItem {
    fn span(&self) -> Span {
        match self {
            MoxItem::Tag(tag) => tag.span(),
            MoxItem::TagNoChildren(tag) => tag.span(),
            MoxItem::Content { span, .. } => *span,
            MoxItem::Fragment(_children) => todo!(),
        }
    }
}

impl From<SnaxItem> for MoxItem {
    fn from(item: SnaxItem) -> Self {
        match item {
            SnaxItem::Tag(t) => MoxItem::Tag(MoxTag::from(t)),
            SnaxItem::SelfClosingTag(t) => MoxItem::TagNoChildren(MoxTagNoChildren::from(t)),
            SnaxItem::Fragment(SnaxFragment { children }) => {
                MoxItem::Fragment(children.into_iter().map(MoxItem::from).collect())
            }
            SnaxItem::Content(atom) => {
                MoxItem::Content { span: atom.span(), stream: wrap_content_tokens(atom) }
            }
        }
    }
}

fn wrap_content_tokens(tt: TokenTree) -> TokenStream {
    let mut new_stream = quote!(#tt);
    match tt {
        TokenTree::Group(g) => {
            let mut tokens = g.stream().into_iter();
            if let Some(TokenTree::Punct(p)) = tokens.next() {
                if p.as_char() == '%' {
                    // strip the percent sign off the front
                    new_stream = TokenStream::new();
                    new_stream.extend(tokens);
                    new_stream = quote!(text(format!(#new_stream)));
                }
            }
        }
        tt @ TokenTree::Ident(_) | tt @ TokenTree::Literal(_) => {
            new_stream = quote!(text(#tt));
        }
        TokenTree::Punct(p) => emit_error!(p.span(), "'{}' not valid in item position", p),
    }
    new_stream
}

impl ToTokens for MoxItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MoxItem::Tag(tag) => tag.to_tokens(tokens),
            MoxItem::TagNoChildren(tag) => tag.to_tokens(tokens),
            MoxItem::Content { stream, .. } => stream.to_tokens(tokens),
            MoxItem::Fragment(_children) => todo!(),
        }
    }
}

struct MoxTag {
    span: Span,
    name: Ident,
    fn_args: Option<MoxArgs>,
    attributes: Vec<MoxAttr>,
    children: Vec<MoxItem>,
}

impl MoxTag {
    fn span(&self) -> Span {
        self.span
    }
}

impl ToTokens for MoxTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tag_to_tokens(&self.name, &self.fn_args, &self.attributes, Some(&self.children), tokens);
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
    // this needs to be nested within other token groups, must be accumulated
    // separately from stream
    let mut contents = quote!();

    for attr in attributes {
        attr.to_tokens(&mut contents);
    }

    if let Some(items) = children {
        for child in items {
            MoxAttr::child(child).to_tokens(&mut contents);
        }
    }

    let fn_args = fn_args.as_ref().map(|args| match &args.value {
        TokenTree::Group(g) => {
            // strip trailing commas that would bork macro parsing
            let mut tokens: Vec<TokenTree> = g.stream().into_iter().collect();

            let last =
                tokens.last().expect("function argument delimiters must contain some tokens");

            if last.to_string() == "," {
                tokens.truncate(tokens.len() - 1);
            }

            let mut without_delim = TokenStream::new();
            without_delim.extend(tokens);
            quote!(#without_delim)
        }
        _ => unimplemented!("bare function args (without a paired delimiter) aren't supported yet"),
    });

    quote!(topo::call(|| { #name(#fn_args) #contents .build() })).to_tokens(stream);
}

impl From<SnaxTag> for MoxTag {
    fn from(SnaxTag { name, attributes, children }: SnaxTag) -> Self {
        let (fn_args, attributes) = args_and_attrs(attributes);
        Self {
            // TODO get the span for the whole tag
            span: name.span(),
            name,
            fn_args,
            attributes,
            children: children.into_iter().map(MoxItem::from).collect(),
        }
    }
}

struct MoxTagNoChildren {
    span: Span,
    name: Ident,
    fn_args: Option<MoxArgs>,
    attributes: Vec<MoxAttr>,
}

impl MoxTagNoChildren {
    fn span(&self) -> Span {
        self.span
    }
}

impl ToTokens for MoxTagNoChildren {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tag_to_tokens(&self.name, &self.fn_args, &self.attributes, None, tokens);
    }
}

impl From<SnaxSelfClosingTag> for MoxTagNoChildren {
    fn from(SnaxSelfClosingTag { name, attributes }: SnaxSelfClosingTag) -> Self {
        let (fn_args, attributes) = args_and_attrs(attributes);
        Self { span: name.span(), name, fn_args, attributes }
    }
}

struct MoxArgs {
    value: TokenTree,
}

struct MoxAttr {
    name: Ident,
    value: TokenStream,
}

impl MoxAttr {
    fn child(item: &MoxItem) -> Self {
        Self { name: Ident::new("child", item.span()), value: item.to_token_stream() }
    }
}

impl ToTokens for MoxAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, value } = self;
        let name = mangle_keywords(name);
        tokens.extend(quote!(.#name(#value)));
    }
}

fn mangle_keywords(name: &Ident) -> Cow<'_, Ident> {
    let replacement = if name == "async" {
        Some("async_")
    } else if name == "for" {
        Some("for_")
    } else if name == "loop" {
        Some("loop_")
    } else if name == "type" {
        Some("type_")
    } else {
        None
    };

    match replacement {
        Some(r) => Cow::Owned(Ident::new(r, name.span())),
        None => Cow::Borrowed(name),
    }
}

impl From<SnaxAttribute> for MoxAttr {
    fn from(attr: SnaxAttribute) -> Self {
        match attr {
            SnaxAttribute::Simple { name, value } => {
                let name_str = name.to_string();
                if name_str == "_" {
                    abort!(
                        name.span(),
                        "anonymous attributes are only allowed in the first position"
                    )
                } else {
                    MoxAttr {
                        name,
                        value: match value {
                            TokenTree::Group(g) => g.stream(),
                            other => other.to_token_stream(),
                        },
                    }
                }
            }
        }
    }
}

enum Error {
    SnaxError(ParseError),
}

impl Into<Diagnostic> for Error {
    fn into(self) -> Diagnostic {
        match self {
            Error::SnaxError(ParseError::UnexpectedEnd) => {
                Diagnostic::new(Level::Error, "input ends before expected".to_string())
            }
            Error::SnaxError(ParseError::UnexpectedItem(item)) => {
                // TODO(#96)
                Diagnostic::new(Level::Error, format!("did not expect {:?}", item))
            }
            Error::SnaxError(ParseError::UnexpectedToken(token)) => {
                Diagnostic::new(Level::Error, format!("did not expect '{}'", token))
            }
        }
    }
}
