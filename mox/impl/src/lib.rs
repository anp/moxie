extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::convert::TryFrom;
use syn::{parse::Parse, parse_macro_input, spanned::Spanned};

#[proc_macro]
pub fn mox(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as MoxItem);
    quote!(#item).into()
}

enum MoxItem {
    Tag(MoxTag),
    Expr(MoxExpr),
}

struct MoxTag {
    name: syn::ExprPath,
    attributes: Vec<MoxAttr>,
    children: Vec<MoxItem>,
}

struct MoxAttr {
    name: syn::Ident,
    value: Option<syn::Expr>,
}

struct MoxExpr {
    expr: syn::Expr,
}

impl Parse for MoxItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parse_config = syn_rsx::ParserConfig::new()
            .number_of_top_level_nodes(1)
            .type_of_top_level_nodes(syn_rsx::NodeType::Element);
        let parser = syn_rsx::Parser::new(parse_config);
        let node = parser.parse(input)?.remove(0);

        MoxItem::try_from(node)
    }
}

impl TryFrom<syn_rsx::Node> for MoxItem {
    type Error = syn::parse::Error;

    fn try_from(node: syn_rsx::Node) -> syn::Result<Self> {
        use syn_rsx::NodeType::*;
        match node.node_type {
            Element => MoxTag::try_from(node).map(MoxItem::Tag),
            Attribute => Err(node_parse_error(&node, "MoxItem")),
            Text | Block => MoxExpr::try_from(node).map(MoxItem::Expr),
        }
    }
}

impl TryFrom<syn_rsx::Node> for MoxTag {
    type Error = syn::parse::Error;

    fn try_from(mut node: syn_rsx::Node) -> syn::Result<Self> {
        use syn_rsx::NodeType::*;
        match node.node_type {
            Element => Ok({
                let attributes: syn::Result<Vec<MoxAttr>> =
                    node.attributes.drain(..).map(|node| MoxAttr::try_from(node)).collect();
                let attributes = attributes?;

                let children: syn::Result<Vec<MoxItem>> =
                    node.children.drain(..).map(|node| MoxItem::try_from(node)).collect();
                let children = children?;

                Self { name: MoxTag::validate_name(node.name.unwrap())?, attributes, children }
            }),
            Attribute | Text | Block => Err(node_parse_error(&node, "MoxTag")),
        }
    }
}

impl MoxTag {
    fn validate_name(name: syn_rsx::NodeName) -> syn::Result<syn::ExprPath> {
        use syn::parse::Error;
        use syn_rsx::NodeName;

        match name {
            NodeName::Path(mut expr_path) => {
                mangle_expr_path(&mut expr_path);
                Ok(expr_path)
            }
            NodeName::Dash(punctuated) => {
                Err(Error::new(punctuated.span(), "Dash tag name syntax isn't supported"))
            }
            NodeName::Colon(punctuated) => {
                Err(Error::new(punctuated.span(), "Colon tag name syntax isn't supported"))
            }
        }
    }
}

fn mangle_expr_path(name: &mut syn::ExprPath) {
    for segment in name.path.segments.iter_mut() {
        mangle_ident(&mut segment.ident);
    }
}

impl TryFrom<syn_rsx::Node> for MoxAttr {
    type Error = syn::parse::Error;

    fn try_from(node: syn_rsx::Node) -> syn::Result<Self> {
        use syn_rsx::NodeType::*;
        match node.node_type {
            Element | Text | Block => Err(node_parse_error(&node, "MoxAttr")),
            Attribute => {
                Ok(MoxAttr { name: MoxAttr::validate_name(node.name.unwrap())?, value: node.value })
            }
        }
    }
}

impl MoxAttr {
    fn validate_name(name: syn_rsx::NodeName) -> syn::Result<syn::Ident> {
        use syn::{parse::Error, punctuated::Pair, PathSegment};
        use syn_rsx::NodeName;

        let invalid_error = |span| Error::new(span, "Invalid name for an attribute");

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
                    _ => Err(invalid_error(segments.span())),
                }
            }
            NodeName::Dash(punctuated) => {
                Err(Error::new(punctuated.span(), "Dash attribute name syntax isn't supported"))
            }
            NodeName::Colon(punctuated) => {
                Err(Error::new(punctuated.span(), "Colon attribute name syntax isn't supported"))
            }
            name => Err(invalid_error(name.span())),
        }
    }
}

fn mangle_ident(ident: &mut syn::Ident) {
    let name = ident.to_string();
    match name.as_str() {
        "async" | "for" | "loop" | "type" => *ident = syn::Ident::new(&(name + "_"), ident.span()),
        _ => (),
    }
}

impl TryFrom<syn_rsx::Node> for MoxExpr {
    type Error = syn::parse::Error;

    fn try_from(node: syn_rsx::Node) -> syn::Result<Self> {
        use syn_rsx::NodeType::*;
        match node.node_type {
            Element | Attribute => Err(node_parse_error(&node, "MoxExpr")),
            Text | Block => Ok(MoxExpr { expr: node.value.unwrap() }),
        }
    }
}

fn node_parse_error(node: &syn_rsx::Node, failed: &'static str) -> syn::parse::Error {
    use syn_rsx::NodeType::*;
    syn::parse::Error::new(node_span(&node), match node.node_type {
        Element => format!("Cannot parse element as a {}", failed),
        Attribute => format!("Cannot parse attribute as a {}", failed),
        Text => format!("Cannot parse text as a {}", failed),
        Block => format!("Cannot parse block as a {}", failed),
    })
}

fn node_span(syn_rsx::Node { name, value, node_type, .. }: &syn_rsx::Node) -> Span {
    use syn_rsx::NodeType::*;
    // TODO get the span for the whole tag, see `https://github.com/rust-lang/rust/issues/54725`
    match node_type {
        Element | Attribute => name.as_ref().unwrap().span(),
        Text | Block => value.as_ref().unwrap().span(),
    }
}

impl ToTokens for MoxItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MoxItem::Tag(tag) => tag.to_tokens(tokens),
            MoxItem::Expr(expr) => expr.to_tokens(tokens),
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
            quote!(.child(#child)).to_tokens(&mut contents);
        }

        quote!(mox::topo::call(|| { #name() #contents .build() })).to_tokens(tokens);
    }
}

impl ToTokens for MoxAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, value } = self;
        let value = match value {
            Some(value) => value.to_token_stream(),
            None => quote! {()},
        };
        tokens.extend(quote!(.#name(#value)));
    }
}

impl ToTokens for MoxExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { expr } = self;
        quote!(#expr.into_child()).to_tokens(tokens);
    }
}
