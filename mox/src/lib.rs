#[macro_use]
extern crate pest_derive;

extern crate proc_macro;
extern crate proc_macro_hack;
extern crate quote;

use self::parser::{Rule, UiParser};
use crate::proc_macro::TokenStream;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use std::{iter::FromIterator, str::FromStr};

mod parser {
    #[derive(Parser)]
    #[grammar = "ui.pest"]
    pub struct UiParser;
}

// fn get_token_stream(mut tag_pairs: Pairs<Rule>) -> TokenStream {
//     let name = tag_pairs.next().expect("name").as_str();
//     let _name = TokenTree::Literal(Literal::string(name));

//     let mut attributes = vec![];
//     let mut handlers = vec![];

//     let vec: Vec<Pair<Rule>> = tag_pairs.next().expect("attributes").into_inner().collect();
//     for i in 0..(vec.len() / 2) {
//         let j = i * 2;
//         let k = &vec[j].as_str();
//         let v = &vec[j + 1];

//         let _v = match v.as_rule() {
//             Rule::embedded => {
//                 let mut _embedded = TokenStream::from_str(v.as_str()).unwrap();
//                 quote!($_embedded.into())
//             }
//             Rule::string => {
//                 let _v = TokenTree::Literal(Literal::string(v.as_str()));
//                 quote! { $_v.into() }
//             }
//             Rule::bool => {
//                 let _v = TokenStream::from_str(v.as_str()).unwrap();
//                 quote! { $_v.into() }
//             }
//             _ => unreachable!(),
//         };

//         if k.starts_with("on") {
//             let (_, k) = k.split_at(2);
//             let _k = TokenTree::Literal(Literal::string(k));
//             handlers.push(quote! {
//                 ($_k.to_string(), _squark::handler($_v)),
//             });
//             continue;
//         }

//         let _k = TokenTree::Literal(Literal::string(k));
//         attributes.push(quote! {
//             ($_k.to_string(), $_v),
//         });
//     }
//     let _attributes = TokenStream::from_iter(attributes);
//     let _handlers = TokenStream::from_iter(handlers);

//     let mut children = vec![];
//     if let Some(children_pair) = tag_pairs.next() {
//         for p in children_pair.into_inner() {
//             let token = match p.as_rule() {
//                 Rule::tag => {
//                     let _tag = get_token_stream(p.into_inner());
//                     quote! {
//                         _squark::Child::from($_tag),
//                     }
//                 }
//                 Rule::text => {
//                     let _text = TokenTree::Literal(Literal::string(p.as_str()));
//                     quote! {
//                         $_text.into(),
//                     }
//                 }
//                 Rule::embedded => {
//                     let _embedded = TokenStream::from_str(p.as_str()).unwrap();
//                     quote! {
//                         {$_embedded}.into(),
//                     }
//                 }
//                 _ => unreachable!(),
//             };
//             children.push(token);
//         }
//     }
//     let _children = TokenStream::from_iter(children);

//     let view = quote! {
//         _squark::View::new(
//             $_name.to_string(),
//             vec![
//                 $_attributes
//             ],
//             vec![
//                 $_handlers
//             ],
//             vec![
//                 $_children
//             ]
//         )
//     };

//     view.into()
// }

#[proc_macro_hack]
pub fn mox(arg: TokenStream) -> TokenStream {
    // let s = arg.to_string();
    // let mut pairs = UiParser::parse(Rule::ui, &s).unwrap();
    // let _token = get_token_stream(pairs.next().unwrap().into_inner());

    // let ret = quote! {
    //     {
    //         extern crate squark as _squark;
    //         $_token
    //     }
    // };

    // ret.into()
    unimplemented!();
}
