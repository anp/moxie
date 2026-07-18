/// To be invoked inside of an `impl TypedValue<'_> for Foo { ... }` item. Expands to an
/// implementation of the `get_members()` method.
macro_rules! declare_members {
    ($register:ident) => {
        fn get_members(&self) -> Option<&'static starlark::environment::Globals> {
            use once_cell::sync::Lazy;
            use starlark::environment::{Globals, GlobalsBuilder};

            static MEMBERS: Lazy<Globals> =
                Lazy::new(|| GlobalsBuilder::new().with($register).build());

            Some(&*MEMBERS)
        }
    };
}

pub mod command;
pub mod formatter;
pub mod json;
pub mod path;
pub mod target;

pub fn register(globals: &mut starlark::environment::GlobalsBuilder) {
    command::register(globals);
    formatter::register(globals);
    json::register(globals);
    path::register(globals);
    target::register(globals);
}
