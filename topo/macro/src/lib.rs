extern crate proc_macro;
use proc_macro::TokenStream;
use syn::export::TokenStream2;

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

    let tmp = std::mem::replace(&mut input_fn.attrs, Vec::new());
    let mut doc_attrs = Vec::new();

    for attr in tmp {
        match attr.parse_meta() {
            Ok(syn::Meta::NameValue(syn::MetaNameValue { ref ident, .. }))
                if ident.to_string() == "doc" =>
            {
                doc_attrs.push(attr);
            }
            _ => input_fn.attrs.push(attr),
        }
    }

    let docs_fn_sig = docs_fn_signature(&input_fn);

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
            doc (
                #(#doc_attrs)*
                #docs_fn_sig
            )
        );

        #[doc(hidden)]
        #input_fn
    )
    .into()
}

/// Creates a series of `#[doc = "..."]` attributes for the input function's signature. This is
/// useful because macros can't have typed return values, arguments, or generics! We embed this
/// output into the final macro output so that it the rendered docs display the type signatures.
fn docs_fn_signature(input_fn: &syn::ItemFn) -> TokenStream2 {
    let doc_fn_sig = syn::ItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        constness: input_fn.constness.clone(),
        asyncness: input_fn.asyncness.clone(),
        unsafety: input_fn.unsafety.clone(),
        abi: input_fn.abi.clone(),
        ident: input_fn.ident.clone(),
        decl: input_fn.decl.clone(),
        block: Box::new(syn::Block {
            brace_token: syn::token::Brace {
                // we
                span: proc_macro::Span::call_site().into(),
            },
            stmts: vec![],
        }),
    };

    let doc_fn_sig = quote::quote!(#doc_fn_sig).to_string();

    use std::io::prelude::*;
    let doc_fn_sig = if let Ok(mut rustfmt) = std::process::Command::new("rustfmt")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        let stdin = rustfmt.stdin.as_mut().unwrap();
        stdin.write_all(doc_fn_sig.as_bytes()).unwrap();

        let output = rustfmt.wait_with_output().unwrap();

        if output.status.success() {
            String::from_utf8(output.stdout).unwrap()
        } else {
            doc_fn_sig
        }
    } else {
        doc_fn_sig
    };

    ["", "# Signature", "", "```ignore", &doc_fn_sig, "```"]
        .iter()
        .map(|&l| quote::quote!(#[doc = #l]))
        .fold(quote::quote!(), |a, b| quote::quote!(#a #b))
}
