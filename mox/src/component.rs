use {
    proc_macro2::{Ident, Span},
    quote::quote,
    syn::{
        parse_macro_input, parse_quote, punctuated::Punctuated, visit_mut::VisitMut, FnArg, FnDecl,
        ItemFn, Macro, Token, TypeParamBound,
    },
};

// TODO convert fn args to `ComponentProps` struct
// TODO make lowercase function name from uppercase component name

pub fn component_impl(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // let comp_decl: ComponentDecl = parse_macro_input!(attrs);
    let mut comp_fn: ItemFn = parse_macro_input!(input);
    validate(&comp_fn);

    let component_name = comp_fn.ident.clone();
    let function_name = crate::component_function_name(&comp_fn.ident);
    let query_group = crate::component_storage_ty_name(&component_name);

    comp_fn.ident = function_name.clone();

    let mut visitor = ComponentVisitor::new(component_name.clone());
    visitor.visit_item_fn_mut(&mut comp_fn);

    quote!(
        #[allow(non_snake_case)]
        #comp_fn

        #[salsa::query_group(#query_group)]
        pub trait #component_name: moxie::Runtime {
            #[allow(non_snake_case)]
            fn #function_name(
                &self,
                scope: moxie::Scope,
                background_color: Color,
                initial_size: Size,
                send_mouse_positions: Sender<CursorMoved>,
            ) -> ();
        }
    )
    .into()
}

fn validate(comp_fn: &ItemFn) {
    assert!(
        comp_fn.unsafety.is_none(),
        "unsafe fns are not supported as components."
    );
    assert!(
        comp_fn.asyncness.is_none(),
        "async fns are not supported as components."
    );
    assert!(
        comp_fn.constness.is_none(),
        "const fns don't make sense to mark as components."
    );
}

struct ComponentVisitor {
    component_name: Ident,
    compose: Ident,
    scope: Ident,
}

impl ComponentVisitor {
    fn new(component_name: Ident) -> Self {
        let scope = Ident::new("scope", Span::call_site());
        let compose = Ident::new("compose", Span::call_site());
        Self {
            compose,
            scope,
            component_name,
        }
    }
}

impl VisitMut for ComponentVisitor {
    fn visit_fn_decl_mut(&mut self, decl: &mut FnDecl) {
        let component_name = &self.component_name;
        let compose = &self.compose;
        let scope = &self.scope;

        let compose_arg: FnArg = parse_quote! { #compose: &impl #component_name };
        let scope_arg: FnArg = parse_quote! { #scope: moxie::Scope };

        decl.inputs.insert(0, compose_arg);
        decl.inputs.insert(1, scope_arg);
    }

    fn visit_macro_mut(&mut self, invocation: &mut Macro) {
        let last_path_segment = &invocation
            .path
            .segments
            .last()
            .unwrap()
            .value()
            .ident
            .to_string();
        let contents = invocation.tts.clone();

        let compose = if last_path_segment == "mox" {
            Some(&self.compose)
        } else {
            None
        };

        let threadables = ["state", "task", "task_fut", "mox", "channel"];
        let (scope, arrow) = if threadables.contains(&last_path_segment.as_str()) {
            let arrow = if last_path_segment == "channel" {
                None
            } else {
                Some(quote!(<-))
            };

            (Some(&self.scope), arrow)
        } else {
            (None, None)
        };

        invocation.tts = parse_quote!(#compose #scope #arrow #contents);
    }
}

#[derive(Debug)]
struct ComponentDecl {
    ident: Ident,
    dependencies: Punctuated<TypeParamBound, Token![,]>,
}

impl syn::parse::Parse for ComponentDecl {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let dependencies: Punctuated<TypeParamBound, Token![,]> =
            input.parse_terminated(TypeParamBound::parse)?;

        Ok(ComponentDecl {
            ident,
            dependencies,
        })
    }
}
