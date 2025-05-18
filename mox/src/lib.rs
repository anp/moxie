//! Library for the `mox!` macro which provides an XML-like syntax for the builder pattern.

extern crate proc_macro;

use proc_macro2::{Punct, TokenStream};
use quote::{quote, ToTokens};
use rstml::node::{
    AttributeValueExpr, CustomNode, KVAttributeValue, KeyedAttribute, KeyedAttributeValue, Node,
    NodeAttribute, NodeBlock, NodeElement, NodeName, NodeNameFragment,
};
use std::convert::TryFrom;
use syn::{
    parse::ParseStream,
    punctuated::{Pair, Punctuated},
    spanned::Spanned,
    token::Comma,
    LitStr,
};

/// Accepts an XML-like expression and expands it to builder-like method calls.
///
/// # Outputs
///
/// The `mox!` macro's contents are expanded to method calls, with `.build()` called on the outmost
/// expression.
///
/// ## Tags
///
/// Each tag expands to a function call with the same name as the tag.
///
/// Each attribute expands to a method called on the value returned from the tag
/// opening or the previous attribute. The attribute name is used as the method
/// name, with the attribute value passed as the argument.
///
/// A tag with children has each child passed as the argument to a call to
/// `.child(...)`, one per child in order of declaration. The calls to `child`
/// come after attributes.
///
/// ## Fragments
///
/// Fragments are not yet supported. See [this issue](https://github.com/anp/moxie/issues/232)
/// for discussion.
///
/// # Inputs
///
/// Each macro invocation must resolve to a single item. Items can be tags,
/// fragments, or content.
///
/// [syn-rsx](https://docs.rs/syn-rsx) is used to tokenize the input as [JSX]\(ish\).
///
/// ## Tags
///
/// Tags always have a name and can have zero or more arguments, attributes, and
/// children.
///
/// They take the form `<NAME ATTR=VAL ...> CHILDREN </NAME>`. Each optional
/// portion can be omitted.
///
/// ### Attributes
///
/// Each attribute takes the form `NAME=VAL` where `NAME` is an identifier and
/// `VALUE` is an expression.
///
/// If the attribute's name is `async`, `for`, `loop`, or `type` an underscore
/// is appended to avoid colliding with the Rust keyword.
///
/// ### Children
///
/// Tags have zero or more nested items (tags, fragments, content) as children.
///
/// If there are no children the tag can be "self-closing": `<NAME ... />`.
///
/// Each child can be either another tag, a Rust literal, or a Rust block (an
/// expression wrapped in `{` and `}`).
///
/// Block expressions can optionally be opened with `{%` to denote a "formatter"
/// item. The enclosed tokens are passed to the `format_args!` macro.
///
/// ### Hyphenated Names
///
/// Tag and attribute names can by hyphenated. The underlying function and
/// method names will have hyphens replaced with underscores. Multiple
/// consecutive hyphens are not supported.
///
/// ## Fragments
///
/// Fragments are opened with `<>` and closed with `</>`. Their only purpose is
/// to provide a parent for children. They do not accept arguments or
/// attributes.
///
/// # Example
///
/// ```
/// use mox::mox;
///
/// #[derive(Debug, PartialEq)]
/// struct Tag {
///     name: String,
///     children: Vec<Tag>,
/// }
///
/// fn built() -> TagBuilder {
///     TagBuilder::default()
/// }
///
/// #[derive(Default)]
/// struct TagBuilder {
///     name: Option<String>,
///     children: Vec<Tag>,
/// }
///
/// impl TagBuilder {
///     fn name(mut self, name: impl Into<String>) -> Self {
///         self.name = Some(name.into());
///         self
///     }
///
///     fn child(mut self, child: TagBuilder) -> Self {
///         self.children.push(child.build());
///         self
///     }
///
///     fn build(self) -> Tag {
///         Tag { name: self.name.unwrap(), children: self.children }
///     }
/// }
///
/// assert_eq!(
///     mox! {
///         <built name="alice">
///             <!-- "This is a comment" -->
///             <built name="bob"/>
///         </built>
///     },
///     Tag {
///         name: String::from("alice"),
///         children: vec![Tag { name: String::from("bob"), children: vec![] }],
///     },
/// );
/// ```
///
/// [JSX]: https://facebook.github.io/jsx/
#[proc_macro]
pub fn mox(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match mox_inner(input.into()) {
        Ok(item) => quote!(#item .build()).into(),
        Err(err) => err.to_compile_error().to_token_stream().into(),
    }
}

fn mox_inner(input: proc_macro2::TokenStream) -> Result<MoxItem, syn::Error> {
    let parse_config =
        rstml::ParserConfig::new().transform_block(parse_fmt_expr).number_of_top_level_nodes(1);
    let parser = rstml::Parser::new(parse_config);
    let node = parser.parse_simple(input)?.remove(0);
    MoxItem::try_from(node)
}

fn parse_fmt_expr(parse_stream: ParseStream) -> syn::Result<Option<TokenStream>> {
    if parse_stream.peek(syn::Token![%]) {
        parse_stream.parse::<syn::Token![%]>()?;
        let arguments: Punctuated<syn::Expr, Comma> =
            Punctuated::parse_separated_nonempty(parse_stream)?;
        if parse_stream.is_empty() {
            Ok(Some(quote!(format_args!(#arguments))))
        } else {
            Err(parse_stream.error(format!("Expected the end, found `{}`", parse_stream)))
        }
    } else {
        Ok(None)
    }
}

enum MoxItem {
    Tag(MoxTag),
    Text(LitStr),
    Block(NodeBlock),
    None,
}

impl<C: CustomNode> TryFrom<Node<C>> for MoxItem {
    type Error = syn::Error;

    fn try_from(node: Node<C>) -> Result<Self, Self::Error> {
        match node {
            Node::Element(elem) => MoxTag::try_from(elem).map(Self::Tag),
            Node::Text(text) => Ok(Self::Text(text.value)),
            Node::Block(block) => Ok(Self::Block(block)),
            // FIXME emit comments for when people expand macros?
            Node::Comment(_) | Node::Doctype(_) => Ok(MoxItem::None),
            Node::Fragment(fragment) => Err(syn::Error::new_spanned(
                fragment,
                "Fragments are not supported as top-level nodes",
            )),
            Node::RawText(text) => {
                Err(syn::Error::new_spanned(text, "Fragments are not supported as top-level nodes"))
            }
            Node::Custom(_) => Err(syn::Error::new_spanned(
                node,
                "Custom nodes are not supported as top-level nodes",
            )),
        }
    }
}

impl ToTokens for MoxItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MoxItem::Tag(tag) => tag.to_tokens(tokens),
            MoxItem::Text(text) => text.to_tokens(tokens),
            MoxItem::Block(block) => {
                quote!(#[allow(unused_braces)]).to_tokens(tokens);
                block.to_tokens(tokens);
            }
            MoxItem::None => (),
        }
    }
}

struct MoxTag {
    name: syn::ExprPath,
    attributes: Vec<MoxAttr>,
    children: Vec<MoxItem>,
}

impl<C> TryFrom<NodeElement<C>> for MoxTag
where
    MoxItem: TryFrom<Node<C>, Error = syn::Error>,
{
    type Error = syn::Error;

    fn try_from(mut elem: NodeElement<C>) -> syn::Result<Self> {
        Ok(Self {
            name: MoxTag::validate_name(elem.open_tag.name)?,
            attributes: elem
                .open_tag
                .attributes
                .drain(..)
                .map(MoxAttr::try_from)
                .collect::<syn::Result<Vec<_>>>()?,
            children: elem
                .children
                .drain(..)
                .map(MoxItem::try_from)
                .collect::<syn::Result<Vec<_>>>()?,
        })
    }
}

fn unsupported_err(
    name: &'static str,
    spanned: impl syn::spanned::Spanned + quote::ToTokens,
) -> syn::Error {
    syn::Error::new_spanned(spanned, format!("{} is not supported", name))
}

impl MoxTag {
    fn validate_name(name: NodeName) -> syn::Result<syn::ExprPath> {
        match name {
            NodeName::Path(mut expr_path) => {
                mangle_expr_path(&mut expr_path);
                Ok(expr_path)
            }
            NodeName::Punctuated(punctuated) => {
                let ident = dashes_to_underscores(punctuated)?;
                let mut segments = Punctuated::new();
                segments.push(ident.into());
                let path = syn::Path { leading_colon: None, segments };

                Ok(syn::ExprPath { attrs: vec![], qself: None, path })
            }
            NodeName::Block(block) => {
                Err(syn::Error::new(block.span(), "Block expression as a tag name isn't supported"))
            }
        }
    }
}

impl ToTokens for MoxTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let MoxTag { name, attributes, children } = self;

        // this needs to be nested within other token groups, must be accumulated
        // separately from stream
        let mut contents = quote!();

        for attr in attributes {
            attr.to_tokens(&mut contents);
        }

        for child in children {
            match child {
                MoxItem::None => (),
                nonempty_child => quote!(.child(#nonempty_child)).to_tokens(&mut contents),
            }
        }

        quote!({ #name() #contents }).to_tokens(tokens);
    }
}

struct MoxAttr {
    name: syn::Ident,
    value: syn::Expr,
}

impl TryFrom<NodeAttribute> for MoxAttr {
    type Error = syn::Error;

    fn try_from(attr: NodeAttribute) -> syn::Result<Self> {
        match attr {
            NodeAttribute::Attribute(KeyedAttribute {
                key,
                possible_value:
                    KeyedAttributeValue::Value(AttributeValueExpr {
                        token_eq: _,
                        value: KVAttributeValue::Expr(value),
                    }),
            }) => Ok(MoxAttr { name: MoxAttr::validate_name(key)?, value }),
            NodeAttribute::Attribute(KeyedAttribute { key: _, possible_value }) => {
                Err(unsupported_err("Non-keyed-expression attribute value", possible_value))
            }
            NodeAttribute::Block(block) => Err(syn::Error::new(
                block.span(),
                "Block expression as an attribute value isn't supported",
            )),
        }
    }
}

impl MoxAttr {
    fn validate_name(name: NodeName) -> syn::Result<syn::Ident> {
        use syn::{punctuated::Pair, PathSegment};

        match name {
            NodeName::Path(syn::ExprPath {
                attrs,
                qself: None,
                path: syn::Path { leading_colon: None, mut segments },
            }) if attrs.is_empty() && segments.len() == 1 => {
                let pair = segments.pop();
                match pair {
                    Some(Pair::End(PathSegment { mut ident, arguments }))
                        if arguments.is_empty() =>
                    {
                        mangle_ident(&mut ident);
                        Ok(ident)
                    }
                    _ => Err(syn::Error::new_spanned(
                        pair,
                        "Single-segment names must not have punctuation",
                    )),
                }
            }
            NodeName::Path(path) => {
                Err(syn::Error::new_spanned(path, "Only single-segment names are supported"))
            }
            NodeName::Punctuated(punctuated) => {
                let ident = dashes_to_underscores(punctuated)?;
                Ok(ident)
            }
            NodeName::Block(block) => Err(syn::Error::new(
                block.span(),
                "Block expression as an attribute name isn't supported",
            )),
        }
    }
}

impl ToTokens for MoxAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, value } = self;
        tokens.extend(quote!(.#name(#value)));
    }
}

fn mangle_expr_path(name: &mut syn::ExprPath) {
    for segment in name.path.segments.iter_mut() {
        mangle_ident(&mut segment.ident);
    }
}

fn mangle_ident(ident: &mut syn::Ident) {
    let name = ident.to_string();
    match name.as_str() {
        "async" | "for" | "loop" | "type" => *ident = syn::Ident::new(&(name + "_"), ident.span()),
        _ => (),
    }
}

fn dashes_to_underscores(
    punctuated: Punctuated<NodeNameFragment, Punct>,
) -> syn::Result<syn::Ident> {
    let mut words = punctuated.pairs();
    let first = words.next().expect("There must be at least one ident in a punctuated list");

    let mut ident_name = match first {
        Pair::Punctuated(first_word, first_punct) => {
            if first_punct.as_char() != '-' {
                return Err(syn::Error::new_spanned(
                    first_punct,
                    "Only hyphenated names are supported",
                ));
            }
            first_word
        }
        Pair::End(first_word) => first_word,
    }
    .to_string();

    for pair in words {
        let w = match pair {
            Pair::Punctuated(w, p) => {
                if p.as_char() != '-' {
                    return Err(syn::Error::new_spanned(p, "Only hyphenated names are supported"));
                }
                w
            }
            Pair::End(w) => w,
        };
        ident_name.push('_');
        ident_name.push_str(&w.to_string());
    }

    if punctuated.trailing_punct() {
        ident_name.push('_');
    }

    Ok(syn::Ident::new(&ident_name, punctuated.span()))
}

#[cfg(test)]
#[test]
fn fails() {
    fn assert_error(input: TokenStream) {
        match mox_inner(input) {
            Ok(_) => unreachable!(),
            Err(error) => println!("{}", error),
        }
    }

    println!();
    assert_error(quote! { <colon:tag:name /> });
    assert_error(quote! { <{"block tag name"} /> });
    assert_error(quote! { <some::tag colon:attribute:name=() /> });
    assert_error(quote! { <some::tag path::attribute::name=() /> });
    assert_error(quote! { {% "1: {}; 2: {}", var1, var2 tail } });
    println!();
}
