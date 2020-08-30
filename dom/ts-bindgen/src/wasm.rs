use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(Debug)]
pub struct WasmBindgenImport {}

impl ToTokens for WasmBindgenImport {
    fn to_tokens(&self, _ts: &mut TokenStream) {
        todo!()
    }
}
