extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::convert::TryFrom;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
};
use syn_rsx::{NodeName, NodeType};

/// Accepts an XML-like expression and expands it to method calls.
///
/// # Outputs
///
/// The `mox!` macro's contents are expanded to function calls, with `.build()` called on the outmost expression.
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
    let item = parse_macro_input!(input as MoxItem);
    quote!(#item .build()).into()
}

enum MoxItem {
    Tag(MoxTag),
    Expr(MoxExpr),
    None,
}

impl Parse for MoxItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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

        let parse_config = syn_rsx::ParserConfig::new()
            .transform_block(parse_fmt_expr)
            .number_of_top_level_nodes(1);
        let parser = syn_rsx::Parser::new(parse_config);
        let node = parser.parse(input)?.remove(0);

        MoxItem::try_from(node)
    }
}

impl TryFrom<syn_rsx::Node> for MoxItem {
    type Error = syn::Error;

    fn try_from(node: syn_rsx::Node) -> syn::Result<Self> {
        match node.node_type {
            NodeType::Element => MoxTag::try_from(node).map(MoxItem::Tag),
            NodeType::Attribute | NodeType::Fragment => Err(Self::node_convert_error(&node)),
            NodeType::Text | NodeType::Block => MoxExpr::try_from(node).map(MoxItem::Expr),
            NodeType::Comment | NodeType::Doctype => Ok(MoxItem::None),
        }
    }
}

impl ToTokens for MoxItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MoxItem::Tag(tag) => tag.to_tokens(tokens),
            MoxItem::Expr(expr) => expr.to_tokens(tokens),
            MoxItem::None => (),
        }
    }
}

struct MoxTag {
    name: syn::ExprPath,
    attributes: Vec<MoxAttr>,
    children: Vec<MoxItem>,
}

impl TryFrom<syn_rsx::Node> for MoxTag {
    type Error = syn::Error;

    fn try_from(mut node: syn_rsx::Node) -> syn::Result<Self> {
        match node.node_type {
            NodeType::Element => Ok(Self {
                name: MoxTag::validate_name(node.name.unwrap())?,
                attributes: node
                    .attributes
                    .drain(..)
                    .map(MoxAttr::try_from)
                    .collect::<syn::Result<Vec<_>>>()?,
                children: node
                    .children
                    .drain(..)
                    .map(MoxItem::try_from)
                    .collect::<syn::Result<Vec<_>>>()?,
            }),
            NodeType::Attribute
            | NodeType::Text
            | NodeType::Block
            | NodeType::Comment
            | NodeType::Doctype
            // TODO(#232) implement
            | NodeType::Fragment => Err(Self::node_convert_error(&node)),
        }
    }
}

impl MoxTag {
    fn validate_name(name: syn_rsx::NodeName) -> syn::Result<syn::ExprPath> {
        match name {
            NodeName::Path(mut expr_path) => {
                mangle_expr_path(&mut expr_path);
                Ok(expr_path)
            }
            NodeName::Dash(punctuated) => {
                // TODO support dash tag name syntax, see `https://github.com/anp/moxie/issues/233`
                Err(syn::Error::new(punctuated.span(), "Dash tag name syntax isn't supported"))
            }
            NodeName::Colon(punctuated) => {
                Err(syn::Error::new(punctuated.span(), "Colon tag name syntax isn't supported"))
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
    value: Option<syn::Expr>,
}

impl TryFrom<syn_rsx::Node> for MoxAttr {
    type Error = syn::Error;

    fn try_from(node: syn_rsx::Node) -> syn::Result<Self> {
        match node.node_type {
            NodeType::Element
            | NodeType::Text
            | NodeType::Block
            | NodeType::Comment
            | NodeType::Doctype
            | NodeType::Fragment => Err(Self::node_convert_error(&node)),
            NodeType::Attribute => {
                Ok(MoxAttr { name: MoxAttr::validate_name(node.name.unwrap())?, value: node.value })
            }
        }
    }
}

impl MoxAttr {
    fn validate_name(name: syn_rsx::NodeName) -> syn::Result<syn::Ident> {
        use syn::{punctuated::Pair, PathSegment};

        let invalid_error = |span| syn::Error::new(span, "Invalid name for an attribute");

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
                    // TODO improve error handling, see `https://github.com/stoically/syn-rsx/issues/12`
                    _ => Err(invalid_error(segments.span())),
                }
            }
            NodeName::Dash(punctuated) => {
                // TODO support dash tag name syntax, see `https://github.com/anp/moxie/issues/233`
                Err(syn::Error::new(
                    punctuated.span(),
                    "Dash attribute name syntax isn't supported",
                ))
            }
            NodeName::Colon(punctuated) => Err(syn::Error::new(
                punctuated.span(),
                "Colon attribute name syntax isn't supported",
            )),
            name => Err(invalid_error(name.span())),
        }
    }
}

impl ToTokens for MoxAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, value } = self;
        match value {
            Some(value) => tokens.extend(quote!(.#name(#value))),
            None => tokens.extend(quote!(.#name(#name))),
        };
    }
}

struct MoxExpr {
    expr: syn::Expr,
}

impl TryFrom<syn_rsx::Node> for MoxExpr {
    type Error = syn::Error;

    fn try_from(node: syn_rsx::Node) -> syn::Result<Self> {
        match node.node_type {
            NodeType::Element
            | NodeType::Attribute
            | NodeType::Comment
            | NodeType::Doctype
            | NodeType::Fragment => Err(Self::node_convert_error(&node)),
            NodeType::Text | NodeType::Block => Ok(MoxExpr { expr: node.value.unwrap() }),
        }
    }
}

// TODO: This produces a warning about unneccessary brackets
impl ToTokens for MoxExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { expr } = self;
        quote!(#expr).to_tokens(tokens);
    }
}

trait NodeConvertError {
    fn node_convert_error(node: &syn_rsx::Node) -> syn::Error {
        syn::Error::new(
            node_span(&node),
            format_args!("Cannot convert {} to {}", node.node_type, std::any::type_name::<Self>(),),
        )
    }
}

impl<T> NodeConvertError for T where T: TryFrom<syn_rsx::Node> {}

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

fn node_span(node: &syn_rsx::Node) -> Span {
    // TODO get the span for the whole node, see `https://github.com/stoically/syn-rsx/issues/14`
    // Prioritize name's span then value's span then call site's span.
    node.name_span()
        .or_else(|| node.value.as_ref().map(|value| value.span()))
        .unwrap_or_else(Span::call_site)
}

#[cfg(test)]
#[test]
fn fails() {
    fn assert_error(input: TokenStream) {
        match syn::parse2::<MoxItem>(input) {
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
