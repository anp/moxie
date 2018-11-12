extern crate gll;
extern crate gll_macros;
extern crate proc_macro;
extern crate proc_macro2;
extern crate proc_macro_hack;
extern crate quote;

use crate::proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_hack::proc_macro_hack;
use quote::quote;

#[proc_macro_hack]
pub fn mox(tokens: TokenStream) -> TokenStream {
    let tokens: TokenStream2 = tokens.into();

    parser::Element::parse_with(tokens, |_parser, result| {
        let parsed = result.unwrap();
    });

    // TODO assert that opening and closing have the same name here

    (quote! { () }).into()
}

mod parser {
    ::gll_macros::proc_macro_parser! {
        Element =
            Bare:SelfClosingElement |
            Full:{ open:OpeningElement children:Child* close:ClosingElement } |
            Frag:Fragment;

        SelfClosingElement = "<"     name:Name attrs:Attribute* "/" ">";
        OpeningElement     = "<"     name:Name attrs:Attribute* ">";
        ClosingElement     = "<" "/" name:Name ">";
        Fragment           = "<" ">" children:Child* "<" "/" ">";

        Name = { IDENT | "::" }* IDENT;

        Attribute = name:Name { "=" value:Value }?;
        Value = Code:Interpolated | Lit:LITERAL | Elem:Element | Frag:Fragment;

        Child = Text:Text | Elem:Element | Frag:Fragment | Code:Interpolated;
        Text = { TextToken+ | Interpolated }+;
        TextToken = Text:IDENT | Lit:LITERAL | Sym:Symbol;
        Symbol =
            "=" | "!" | "~" | "+" |
            "-" | "*" | "/" | "%" |
            "^" | "&" | "|" | "@" |
            "." | "," | ";" | ":" |
            "#" | "$" | "?" | "(" |
            ")";

        // TODO support other types of interpolation syntax?
        Interpolated = "{" TOKEN_TREE+ "}";
    }
}
