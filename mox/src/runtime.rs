use {
    quote::quote,
    syn::{
        parse_macro_input, punctuated::Punctuated, spanned::Spanned, Ident, ItemStruct, Token,
        TypeParamBound,
    },
};

pub(crate) fn runtime_impl(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let runtime_decl: RuntimeDecl = parse_macro_input!(attrs);
    let mut runtime_struct: ItemStruct = parse_macro_input!(input);
    let runtime_name = runtime_struct.ident.clone();
    let dependencies = &runtime_decl.dependencies;

    let mut new_fields: syn::FieldsNamed = match runtime_struct.fields {
        syn::Fields::Unnamed(_) => {
            runtime_struct
                .fields
                .span()
                .unwrap()
                .error("tuple structs not yet supported")
                .emit();
            unreachable!()
        }
        syn::Fields::Named(ref fields) => fields.clone(),
        syn::Fields::Unit => syn::parse_quote!({}),
    };

    let to_merge: syn::FieldsNamed = syn::parse_quote!({
        salsa_rt: salsa::Runtime<#runtime_name>,
        scopes: moxie::Scopes,
    });
    new_fields.named.extend(to_merge.named);
    runtime_struct.fields = syn::Fields::Named(new_fields);

    let mut storage_dependencies = dependencies.clone();
    for dep in &mut storage_dependencies {
        // TODO replace last path element with crate::component_storage_ty_name
        match dep {
            syn::TypeParamBound::Lifetime(_) => dep
                .span()
                .unwrap()
                .error("database constraints must be 'static anyways, no lifetimes allowed here.")
                .emit(),
            syn::TypeParamBound::Trait(trait_bound) => {
                let last_segment = &mut trait_bound.path.segments.last_mut().unwrap();
                let trait_name = &mut last_segment.value_mut().ident;

                let component_dependency = crate::component::Name::new(trait_name.clone());

                *trait_name = component_dependency.storage_ty();
            }
        }
    }

    quote!(
        #[salsa::database( moxie::TaskStorage, #storage_dependencies )]
        #runtime_struct

        impl salsa::Database for #runtime_name {
            fn salsa_runtime(&self) -> &salsa::Runtime<Self> {
                &self.salsa_rt
            }
        }

        impl moxie::Runtime for #runtime_name {
            fn scope(&self, id: moxie::ScopeId) -> moxie::Scope {
                self.scopes.get(id, self)
            }
        }
    )
    .into()
}

#[derive(Debug)]
struct RuntimeDecl {
    trait_name: Ident,
    dependencies: Punctuated<TypeParamBound, Token![,]>,
}

impl syn::parse::Parse for RuntimeDecl {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let trait_name = input.parse()?;

        let dependencies: Punctuated<TypeParamBound, Token![,]> =
            if input.parse::<Token![:]>().is_ok() {
                input.parse_terminated(TypeParamBound::parse)?
            } else {
                Punctuated::new()
            };
        Ok(RuntimeDecl {
            trait_name,
            dependencies,
        })
    }
}
