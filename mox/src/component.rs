use {
    proc_macro2::{Ident, Span},
    quote::quote_spanned,
    syn::{
        parse2 as parse, punctuated::Punctuated, spanned::Spanned, visit_mut::VisitMut, Field,
        FnArg, FnDecl, ItemFn, ItemMod, ItemStruct, ItemTrait, Macro, Path, Stmt, Token,
        TypeParamBound,
    },
};

mod parse;

#[derive(Debug)]
pub struct ComponentMacro {
    comp_fn: ItemFn,
    name: Name,
    fields: Punctuated<Field, Token![,]>,
    field_names: Punctuated<Ident, Token![,]>,
    dependencies: Punctuated<TypeParamBound, Token![+]>,
}

impl ComponentMacro {
    pub fn expand(&self) -> ItemMod {
        let mod_name = self.name.mod_name();
        let component_function = self.comp_fn.clone();
        let props_type_decl = self.props_type_decl();
        let query_decl = self.query_decl();
        let test_runtime_decl = self.test_runtime_decl();

        let decl = quote::quote!(
            pub mod #mod_name {
                use super::*;
                #component_function
                #props_type_decl
                #query_decl
                #test_runtime_decl
            }
        );

        parse(decl).unwrap()
    }

    fn props_type_decl(&self) -> ItemStruct {
        let props_type = self.name.props_ty_ident();
        let fields = &self.fields;
        parse(quote_spanned!(self.args_span()=>
            #[derive(Clone, Debug, Eq, Hash, PartialEq)]
            pub struct #props_type {
                #fields
            }
        ))
        .unwrap()
    }

    fn props_destructure(&self) -> Stmt {
        let props = self.props_ident();
        let props_type = self.name.props_ty_ident();
        let field_names = &self.field_names;
        parse(quote_spanned!(self.args_span()=> let #props_type { #field_names } = #props;))
            .unwrap()
    }

    fn query_decl(&self) -> ItemTrait {
        let function_name = self.name.fn_name();
        let query_name = self.name.query_trait_ident();
        let storage_ty = self.name.storage_ty_ident();
        let props_type = self.name.props_ty_ident();
        let dependencies = &self.dependencies;
        let maybe_colon: Option<Token![:]> = if !dependencies.is_empty() {
            Some(parse(quote::quote!(:)).unwrap())
        } else {
            None
        };

        let decl = quote::quote!(
            #[salsa::query_group(#storage_ty)]
            pub trait #query_name #maybe_colon #dependencies {
                fn #function_name(
                    &self,
                    scope: moxie::Scope,
                    props: #props_type
                ) -> ();
            }
        );

        parse(decl).unwrap()
    }

    fn test_runtime_decl(&self) -> ItemStruct {
        let query_trait = self.name.query_trait_ident();
        let dependencies = &self.dependencies;
        let test_runtime_name = self.name.test_runtime_ty_ident();
        let maybe_plus: Option<Token![+]> = if !dependencies.is_empty() {
            Some(parse(quote::quote!(+)).unwrap())
        } else {
            None
        };
        parse(quote::quote!(
            #[moxie::runtime(#query_trait #maybe_plus #dependencies)]
            struct #test_runtime_name;
        ))
        .unwrap()
    }

    fn props_ident(&self) -> Ident {
        Ident::new("props", Span::call_site())
    }

    fn scope_ident(&self) -> Ident {
        Ident::new("scope", Span::call_site())
    }

    fn compose_ident(&self) -> Ident {
        Ident::new("compose", Span::call_site())
    }

    fn args_span(&self) -> Span {
        self.comp_fn.decl.inputs.span()
    }

    fn threader(&self) -> BlockThreader {
        BlockThreader {
            component_name: self.name.clone(),
            compose: self.compose_ident(),
            scope: self.scope_ident(),
            props: self.props_ident(),
            args_span: self.args_span(),
            props_destructure: self.props_destructure(),
        }
    }
}

struct BlockThreader {
    component_name: Name,
    compose: Ident,
    scope: Ident,
    props: Ident,
    args_span: Span,
    props_destructure: Stmt,
}

impl VisitMut for BlockThreader {
    fn visit_item_fn_mut(&mut self, comp_fn: &mut ItemFn) {
        syn::visit_mut::visit_item_fn_mut(self, comp_fn);
        comp_fn
            .block
            .stmts
            .insert(0, self.props_destructure.clone());
    }

    /// Replace the existing function's arguments with the concrete component signature of
    /// `(compose, scope, props)`.
    fn visit_fn_decl_mut(&mut self, decl: &mut FnDecl) {
        let Self {
            component_name,
            compose,
            scope,
            props,
            args_span,
            ..
        } = self;

        let composer_trait = component_name.query_trait();
        let props_type = component_name.props_ty();

        let compose_field: FnArg =
            parse(quote_spanned! {*args_span=> #compose: &impl #composer_trait }).unwrap();
        let scope_field: FnArg =
            parse(quote_spanned! {*args_span=> #scope: moxie::Scope }).unwrap();
        let props_field: FnArg = parse(quote_spanned! {*args_span=> #props: #props_type }).unwrap();

        use std::iter::FromIterator;
        decl.inputs = Punctuated::from_iter(vec![compose_field, scope_field, props_field]);

        syn::visit_mut::visit_fn_decl_mut(self, decl);
    }

    /// "Threads" the `compose` and `scope` identifiers through the various macro invocations where
    /// they are required arguments.
    ///
    /// TODO: expand macros recursively
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
                Some(quote_spanned!(last_path_segment.span()=> <-))
            };

            (Some(&self.scope), arrow)
        } else {
            (None, None)
        };

        // FIXME handle nested macro invocations
        invocation.tts =
            parse(quote_spanned!(invocation.tts.span()=> #compose #scope #arrow #contents))
                .unwrap();
        syn::visit_mut::visit_macro_mut(self, invocation);
    }
}

#[derive(Clone, Debug)]
pub struct Name(Path);

impl Name {
    const COMPOSER: &'static str = "Composer";
    const PROPS: &'static str = "Props";
    const STORAGE: &'static str = "Storage";
    const TEST_RUNTIME: &'static str = "TestRuntime";

    pub fn new(module_path: Path) -> Self {
        Name(module_path)
    }

    pub fn fn_name(&self) -> Ident {
        use inflector::Inflector;
        let snaked = self.mod_name().to_string().to_snake_case();
        Ident::new(&snaked, self.0.span())
    }

    pub fn mod_name(&self) -> Ident {
        self.0.segments.last().unwrap().value().ident.clone()
    }

    pub fn query_trait(&self) -> Path {
        self.friend_ty_path(Self::COMPOSER)
    }

    pub fn query_trait_ident(&self) -> Ident {
        self.spanned_ident(Self::COMPOSER)
    }

    pub fn props_ty(&self) -> Path {
        self.friend_ty_path(Self::PROPS)
    }

    pub fn props_ty_ident(&self) -> Ident {
        self.spanned_ident(Self::PROPS)
    }

    pub fn storage_ty(&self) -> Path {
        self.friend_ty_path(Self::STORAGE)
    }

    pub fn storage_ty_ident(&self) -> Ident {
        self.spanned_ident(Self::STORAGE)
    }

    pub fn test_runtime_ty_ident(&self) -> Ident {
        self.spanned_ident(Self::TEST_RUNTIME)
    }

    fn spanned_ident(&self, contents: &str) -> Ident {
        let mut name_str = self.mod_name().to_string();
        name_str.push_str(contents);
        Ident::new(&name_str, self.0.span())
    }

    fn friend_ty_path(&self, friend: &str) -> Path {
        let mut copied = self.0.clone();
        copied.segments.push(self.spanned_ident(friend).into());
        copied
    }
}
