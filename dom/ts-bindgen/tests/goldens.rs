use pretty_assertions::assert_eq;
use std::path::Path;
use ts_bindgen::make_bindings;

const GOLDENS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/goldens/");

macro_rules! golden_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let goldens = Path::new(GOLDENS_DIR);
            let input_path = goldens.join(concat!(stringify!($name), ".d.ts"));
            let expect_path = goldens.join(concat!(stringify!($name), ".rs"));

            let input = std::fs::read_to_string(input_path).expect("reading test input");
            let expected = std::fs::read_to_string(expect_path).expect("reading expected output");

            let actual = make_bindings(&input).expect("generating bindings");
            assert_eq!(actual, expected);
        }
    };
}
