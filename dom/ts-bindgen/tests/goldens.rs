use pretty_assertions::assert_eq;
use ts_bindgen::make_bindings;

macro_rules! golden {
    ($($file:expr),+) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/goldens/", $($file),+)
    };
}

macro_rules! golden_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let input_path = golden!(stringify!($name), ".d.ts");
            let expect_path = golden!(stringify!($name), ".rs");
            mod make_sure_expected_output_builds {
                include!(golden!(stringify!($name), ".rs"));
            }

            let input = std::fs::read_to_string(input_path).expect("reading test input");
            let expected = std::fs::read_to_string(expect_path)
                .expect("reading expected output")
                .split("// @@ end-expected @@ //")
                .next()
                .unwrap()
                .to_string();

            let actual = make_bindings(&input).expect("generating bindings");
            assert_eq!(actual, expected);
        }
    };
}

golden_test!(bare_function);
