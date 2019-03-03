use {
    proc_macro2::{Ident, Span},
    quote::quote,
    syn::{parse_macro_input, parse_quote, visit_mut::VisitMut, FnArg, FnDecl, ItemFn, Macro},
};

pub fn component_impl(
    _attrs: proc_macro::TokenStream, // TODO customize Components trait, other potential arguments?
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut comp_fn: ItemFn = parse_macro_input!(input);

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

    let mut visitor = ComponentVisitor::new(None);
    visitor.visit_item_fn_mut(&mut comp_fn);

    quote!( #comp_fn ).into()
}

struct ComponentVisitor {
    components_trait: Ident,
    compose: Ident,
    scope: Ident,
}

impl ComponentVisitor {
    fn new(components_trait: Option<Ident>) -> Self {
        let scope = Ident::new("scope", Span::call_site());
        let compose = Ident::new("compose", Span::call_site());

        let components_trait =
            components_trait.unwrap_or_else(|| Ident::new("Components", Span::call_site()));
        Self {
            components_trait,
            compose,
            scope,
        }
    }
}

impl VisitMut for ComponentVisitor {
    fn visit_fn_decl_mut(&mut self, decl: &mut FnDecl) {
        let compose = &self.compose;
        let components_trait = &self.components_trait;
        let scope = &self.scope;

        let compose_arg: FnArg = parse_quote! { #compose: &impl #components_trait };
        let scope_arg: FnArg = parse_quote! { #scope: moxie::Scope };

        decl.inputs.insert(0, compose_arg);
        decl.inputs.insert(1, scope_arg);
    }

    fn visit_macro_mut(&mut self, invocation: &mut Macro) {
        // TODO check if the macro is on our list of "scope-threadables"
        // TODO accept `moxie::MACRO!` or `MACRO!`?
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
