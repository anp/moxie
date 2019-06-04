extern crate proc_macro;
use proc_macro::TokenStream;

/// Topological functions form a runtime graph from their control flow, and each function's
/// scope has a unique identifier at runtime based on that specific invocation's position within
/// this call graph.
///
/// ## Implications of using macros
///
/// This has a number of implications: it is not currently feasible to abstract over topological
/// functions, visibility rules are less granular, and error messages are generally worse. Because
/// topological function calls are a bit more expensive than normal function calls, it makes sense
/// to call them out as different invocations than typical functions, in particular because they
/// literally bind to the source location at which they're invoked.
///
/// ## Macro alternatives
///
/// ### React: top-level of functions only
///
/// TODO explain more
///
/// ### Compose: uses nifty compiler transform on each callsite
///
/// TODO
///
/// ### Rust #\[track_caller\] attribute
///
/// Not yet implemented, not clear it'll offer same guarantees as chained TypeIds.
///
/// TODO expand
#[proc_macro_attribute]
pub fn topo(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn: syn::ItemFn = syn::parse_macro_input!(input);

    let mangled_name = syn::Ident::new(
        &format!("__{}_impl", &input_fn.ident),
        input_fn.ident.span(),
    );

    let macro_name = std::mem::replace(&mut input_fn.ident, mangled_name.clone());

    quote::quote!(
        topo::__make_topo_macro!(
            #macro_name #mangled_name
            match ($($arg:expr),*)
            subst ($($arg),*)
        );

        #input_fn
    )
    .into()
}
