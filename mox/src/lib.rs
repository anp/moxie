extern crate proc_macro;

use proc_macro2::{Delimiter, Group, Ident, TokenStream, TokenTree};
use proc_macro_error::{abort, emit_error, proc_macro_error, Diagnostic, Level, ResultExt};
use proc_macro_hack::proc_macro_hack;
use quote::{quote, ToTokens};
use snax::{ParseError, SnaxAttribute, SnaxFragment, SnaxItem, SnaxSelfClosingTag, SnaxTag};

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
                    new_stream = quote!(text(format!(#new_stream)));
                }
            }
        }
        tt @ TokenTree::Ident(_) | tt @ TokenTree::Literal(_) => {
            new_stream = quote!(text(#tt));
        }
        TokenTree::Punct(p) => emit_error!(p.span(), "'{}' not valid in item position", p),
    }
    Group::new(Delimiter::Brace, new_stream).into()
}

impl ToTokens for MoxItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MoxItem::Tag(tag) => tag.to_tokens(tokens),
            MoxItem::TagNoChildren(tag) => tag.to_tokens(tokens),
            MoxItem::Fragment(children) => tokens.extend(quote!({ #(#children;)* })),
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

    attributes.iter().map(ToTokens::to_token_stream).for_each(|ts| contents.extend(ts));

    if let Some(items) = children {
        let mut children = quote!();
        items.iter().map(ToTokens::to_token_stream).for_each(|ts| children.extend(quote!(#ts;)));

        contents.extend(quote!(
            .inner(|| {
                #children
            })
        ));
    } else if !contents.is_empty() {
        // if there were attributes or handlers installed but there isn't an inner
        // function to call with its own return type, the previous calls
        // probably return references that can't return
        contents.extend(quote!(;));
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

    let invocation = if contents.is_empty() {
        quote!(#name(#fn_args))
    } else {
        if fn_args.is_some() {
            unimplemented!(
                "can't emit function arguments at the same time as attributes or children yet"
            )
        }
        quote!(topo::call(|| { #name() #contents }))
    };

    stream.extend(invocation);
}

impl From<SnaxTag> for MoxTag {
    fn from(SnaxTag { name, attributes, children }: SnaxTag) -> Self {
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
        Self { name, fn_args, attributes }
    }
}

struct MoxArgs {
    value: TokenTree,
}

struct MoxAttr {
    name: Ident,
    value: TokenTree,
}

impl ToTokens for MoxAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, value } = self;
        tokens.extend(if name == "type" { quote!(.ty(#value)) } else { quote!(.#name(#value)) });
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
                    MoxAttr { name, value }
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
