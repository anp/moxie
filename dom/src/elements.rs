macro_rules! element_macro {
    ($name:ident) => {
        #[macro_export]
        macro_rules! $name {
            ($with_elem:expr) => {
                $crate::element!(stringify!($name), $with_elem)
            };
        }
    };
}

element_macro!(a);
element_macro!(button);
element_macro!(div);
element_macro!(footer);
element_macro!(h1);
element_macro!(header);
element_macro!(input);
element_macro!(label);
element_macro!(li);
element_macro!(section);
element_macro!(span);
element_macro!(strong);
element_macro!(ul);
