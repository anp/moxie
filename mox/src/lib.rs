extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn mox(input: TokenStream) -> TokenStream {
    let _ = input;

    unimplemented!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
