extern crate mox;
extern crate proc_macro;
extern crate proc_macro2;

use crate::proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub fn mox(tokens: TokenStream) -> TokenStream {
    mox::parse_tokens_and_declare(tokens.into()).into()
}
