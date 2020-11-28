use starlark::environment::{Environment, TypeValues};

// TODO delete this
macro_rules! starlark_module {
    ($($tok:tt)+) => {
        // TODO submit an upstream patch to use $crate in all these macros
        use starlark::{
            starlark_fun,
            starlark_module as raw_starlark_module,
            starlark_parse_param_type,
            starlark_signature,
            starlark_signature_extraction,
            starlark_signatures,
        };

        raw_starlark_module! {$($tok)+}
    };
}

pub mod command;
pub mod formatter;
pub mod json;
pub mod target;

pub fn register(env: &mut Environment, tvs: &mut TypeValues) {
    command::globals(env, tvs);
    formatter::globals(env, tvs);
    json::globals(env, tvs);
    target::globals(env, tvs);
}
