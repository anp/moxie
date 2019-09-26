extern crate proc_macro;

use {
    proc_macro::TokenStream,
    proc_macro_error::{filter_macro_errors, MacroError, ResultExt},
    snax::{ParseError, SnaxItem},
};

#[proc_macro_hack::proc_macro_hack]
pub fn moxml(input: TokenStream) -> TokenStream {
    filter_macro_errors! {
        let item = snax::parse(input.into()).map_err(Error::SnaxError).unwrap_or_exit();
        declare_elements(item).unwrap_or_exit()
    }
}

fn declare_elements(item: SnaxItem) -> Result<TokenStream, Error> {
    unimplemented!()
}

enum Error {
    SnaxError(ParseError),
}

impl Into<MacroError> for Error {
    fn into(self) -> MacroError {
        match self {
            Error::SnaxError(ParseError::UnexpectedEnd) => {
                MacroError::call_site(format!("input ends before expected"))
            }
            Error::SnaxError(ParseError::UnexpectedItem(item)) => {
                // TODO https://github.com/LPGhatguy/snax/issues/9
                MacroError::call_site(format!("did not expect {:?}", item))
            }
            Error::SnaxError(ParseError::UnexpectedToken(token)) => {
                MacroError::new(token.span(), format!("did not expect '{}'", token))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
