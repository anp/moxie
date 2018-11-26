extern crate gll;
extern crate gll_macros;
extern crate proc_macro2;
extern crate quote;

use proc_macro2::TokenStream;
use quote::quote;

pub fn parse_tokens_and_declare(tokens: TokenStream) -> TokenStream {
    let input = tokens.to_string();
    unimplemented!();

    // let mut declared = None;

    // Element::parse_with(tokens, |_parser, result| {
    //     let element = result.unwrap().one().unwrap();
    //     declared = Some(element.declare());
    // });

    // declared.unwrap().into()
}

// trait Declare {
//     fn declare(&self) -> TokenStream;
// }

// impl<'a, 'b, T: gll::runtime::Input> Declare for Element<'a, 'b, T> {
//     fn declare(&self) -> TokenStream {
//         match self {
//             Element::Bare(self_closing) => self_closing.one().unwrap().declare(),
//             Element::Full(full) => full.one().unwrap().declare(),
//             Element::Frag(_) => panic!("Fragments are not yet supported."),
//         }
//     }
// }

// impl<'a, 'b, T: gll::runtime::Input> Declare for SelfClosingElement<'a, 'b, T> {
//     fn declare(&self) -> TokenStream {
//         unimplemented!();
//     }
// }

// impl<'a, 'b, T: gll::runtime::Input> Declare for FullElement<'a, 'b, T> {
//     fn declare(&self) -> TokenStream {
//         let open = self.open.one().unwrap();
//         let close = self.close.one().unwrap();

//         let name = open.name.source();

//         // if name != close.name.source() {
//         //     panic!("Opening and closing tags must have the same name.");
//         // }

//         // name

//         // attrs

//         // handlers?

//         // children

//         quote! {
//             #name {

//             }.create(vec![

//             ])
//         }
//     }
// }
// let macrod = mox! {
//     <View id="container">
//         <Input value="foo" _type="text"/>
//         <Link href="/bar"/>
//         <Text>hello world now</Text>
//     </View>
// };

// let manual = View {
//     id: "container".to_owned().into(),
// }
// .create(vec![
//     Input {
//         value: "foo".into(),
//         _type: "text".into(),
//     }
//     .create(vec![]),
//     Link {
//         href: "/bar".into(),
//     }
//     .create(vec![]),
//     Text.create(vec![TextNode("hello world now".into()).create(vec![])]),
// ]);

// impl<'a, 'b, T: gll::runtime::Input> Declare for parser::Element<'a, 'b, T> {
//     fn declare(&self) -> TokenStream {
//         unimplemented!();
//     }
// }
::gll_macros::proc_macro_parser! {
    Element =
        Bare:SelfClosingElement |
        Full:FullElement |
        Frag:Fragment;

    SelfClosingElement = "<"     name:Name attrs:Attribute* "/" ">";
    FullElement        = open:OpeningElement children:Child* close:ClosingElement;
    OpeningElement     = "<"     name:Name attrs:Attribute* ">";
    ClosingElement     = "<" "/" name:Name ">";
    Fragment           = "<" ">" children:Child* "<" "/" ">";

    Name = components:IDENT+ % "::";

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

    Interpolated = "{" TOKEN_TREE+ "}";
}
