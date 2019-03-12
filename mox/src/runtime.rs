use {
    quote::quote,
    syn::{
        parse_macro_input, punctuated::Punctuated, spanned::Spanned, ItemStruct, Token,
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

    quote!(
        #[salsa::database( #dependencies )]
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
    dependencies: Punctuated<TypeParamBound, Token![,]>,
}

impl syn::parse::Parse for RuntimeDecl {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let dependencies: Punctuated<TypeParamBound, Token![+]> =
            input.parse_terminated(TypeParamBound::parse)?;

        let dependencies = dependencies
            .into_iter()
            .map(|mut bound| {
                match &mut bound {
                    TypeParamBound::Lifetime(lt) => lt
                        .span()
                        .unwrap()
                        .error("runtime data must be 'static")
                        .emit(),
                    TypeParamBound::Trait(trt) => {
                        let name = crate::component::Name::new(trt.path.clone());
                        trt.path = name.storage_ty();
                    }
                }
                bound
            })
            .collect();

        Ok(RuntimeDecl { dependencies })
    }
}
