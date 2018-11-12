extern crate proc_macro_hack;

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use mox_macro::mox;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
