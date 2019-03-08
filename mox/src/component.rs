use {
    failure::*,
    proc_macro2::{Ident, Span},
    quote::quote_spanned,
    syn::{
        parse2 as parse, punctuated::Punctuated, spanned::Spanned, visit_mut::VisitMut, Block,
        Field, FieldsNamed, FnArg, FnDecl, Item, ItemFn, ItemStruct, Macro, Stmt, Token,
        TypeParamBound,
    },
};

pub struct ComponentMacro {
    comp_fn: ItemFn,
    comp_decl: Option<ComponentDecl>,
    name: Name,
    fields: Punctuated<Field, Token![,]>,
    field_names: Punctuated<Ident, Token![,]>,
    annotation_span: Span,
}

impl ComponentMacro {
    pub fn new(
        comp_decl: Option<ComponentDecl>,
        comp_decl_span: Span,
        input_fn: ItemFn,
    ) -> Result<Self, Error> {
        let name = Name::new(input_fn.ident.clone());
        ensure!(
            input_fn.unsafety.is_none(),
            "unsafe fns are not supported as components."
        );
        ensure!(
            input_fn.asyncness.is_none(),
            "async fns are not supported as components."
        );
        ensure!(
            input_fn.constness.is_none(),
            "const fns don't make sense to mark as components."
        );

        let (mut fields, mut field_names): (
            Punctuated<Field, Token![,]>,
            Punctuated<Ident, Token![,]>,
        ) = (Punctuated::new(), Punctuated::new());

        for arg in &input_fn.decl.inputs {
            let mut field: FieldsNamed = parse(quote_spanned!(arg.span()=> { #arg })).unwrap();
            let field: Field = field.named.pop().unwrap().into_value();
            field_names.push(field.ident.clone().unwrap());
            fields.push(field);
        }

        let mut comp_macro = ComponentMacro {
            name,
            comp_fn: input_fn.clone(),
            comp_decl,
            fields,
            field_names,
            annotation_span: comp_decl_span,
        };

        // uh this is silly and also appears to not work?
        let mut new_comp_fn = input_fn.clone();
        comp_macro.visit_item_fn_mut(&mut new_comp_fn);
        comp_macro.comp_fn = new_comp_fn;

        Ok(comp_macro)
    }

    fn props_type_decl(&self) -> ItemStruct {
        let props_type = self.name.props_ty();
        let fields = &self.fields;
        parse(quote_spanned!(self.args_span()=>
            #[derive(Clone, Debug, Eq, Hash, PartialEq)]
            pub struct #props_type {
                #fields
            }
        ))
        .unwrap()
    }

    pub fn props_destructure(&self) -> Stmt {
        let props = self.props_ident();
        let props_type = self.name.props_ty();
        let field_names = &self.field_names;
        parse(quote_spanned!(self.args_span()=> let #props_type { #field_names } = #props;))
            .unwrap()
    }

    fn props_ident(&self) -> Ident {
        Ident::new("props", self.annotation_span)
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

    pub fn expand(&self) -> Vec<Item> {
        let comp_fn: &ItemFn = &self.comp_fn;

        let function_name = self.name.fn_name();
        let query_name = self.name.query_trait();
        let query_group = self.name.storage_ty();
        let test_runtime_name = self.name.test_runtime_ty();
        let props_type = self.name.props_ty();
        let props_type_decl = self.props_type_decl();

        let query_decl = quote_spanned!(self.annotation_span=>
            #[salsa::query_group(#query_group)]
            pub trait #query_name: moxie::Runtime {
            // pub trait #query_name: moxie::Runtime #query_deps {
                fn #function_name(
                    &self,
                    scope: moxie::Scope,
                    props: #props_type
                ) -> ();
            }
        );

        let test_runtime_decl = quote_spanned!(self.annotation_span=>
            #[moxie::runtime(#query_group)]
            // #[moxie::runtime(#query_group #query_group_deps)]
            struct #test_runtime_name;
        );

        vec![
            From::from(comp_fn.clone()),
            From::from(props_type_decl),
            parse(query_decl).unwrap(),
            parse(test_runtime_decl).unwrap(),
        ]
    }
}

impl VisitMut for ComponentMacro {
    /// Replace the existing function's arguments with the concrete component signature of
    /// `(compose, scope, props)`.
    fn visit_fn_decl_mut(&mut self, decl: &mut FnDecl) {
        let component_name = self.name.fn_name();
        let props_type = self.name.props_ty();
        let compose = self.compose_ident();
        let scope = self.scope_ident();
        let props = self.props_ident();

        let compose_field: FnArg =
            parse(quote_spanned! {component_name.span()=> #compose: &impl #component_name })
                .unwrap();
        let scope_field: FnArg =
            parse(quote_spanned! {component_name.span()=> #scope: moxie::Scope }).unwrap();
        let props_field: FnArg =
            parse(quote_spanned! {self.args_span()=> #props: #props_type }).unwrap();

        use std::iter::FromIterator;
        decl.inputs = Punctuated::from_iter(vec![compose_field, scope_field, props_field]);
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
            Some(self.compose_ident())
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

            (Some(self.scope_ident()), arrow)
        } else {
            (None, None)
        };

        invocation.tts =
            parse(quote_spanned!(invocation.tts.span()=> #compose #scope #arrow #contents))
                .unwrap();
    }

    /// We visit the block of the component function to insert a destructuring of props that we
    /// had previously bundled into a single argument type.
    fn visit_block_mut(&mut self, block: &mut Block) {
        block.stmts.insert(0, self.props_destructure());
    }
}

#[derive(Debug)]
pub struct ComponentDecl {
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

pub struct Name(Ident);

impl Name {
    pub fn new(n: Ident) -> Self {
        Name(n)
    }

    pub fn fn_name(&self) -> Ident {
        self.0.clone()
    }

    pub fn query_trait(&self) -> Ident {
        self.friend_type("Composer")
    }

    pub fn props_ty(&self) -> Ident {
        self.friend_type("Props")
    }

    pub fn storage_ty(&self) -> Ident {
        self.friend_type("Storage")
    }

    pub fn test_runtime_ty(&self) -> Ident {
        self.friend_type("TestRuntime")
    }

    fn friend_type(&self, suffix: &str) -> Ident {
        let mut comp_name_str = self.0.to_string();
        comp_name_str.push_str(suffix);
        Ident::new(&comp_name_str, self.0.span())
    }
}
