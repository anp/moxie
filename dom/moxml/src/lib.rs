extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn moxml(input: TokenStream) -> TokenStream {
    let item = snax::parse(input.into()).unwrap();

    unimplemented!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
