use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn updater(name: TokenStream, input: TokenStream) -> TokenStream {
    let name: syn::Ident = syn::parse(name).unwrap();
    let impls: syn::ItemImpl = syn::parse(input).unwrap();

    let state_ty = impls.self_ty.clone();

    let mut updater_items = vec![];

    for item in &impls.items {
        if let syn::ImplItem::Method(method) = item {
            if let Some(updater_method) = make_updater_method(method) {
                updater_items.push(syn::ImplItem::Method(updater_method));
            }
        }
    }

    let updater_doc = format!("A wrapper around `moxie::Key<{}>.", quote!(#state_ty));

    quote!(
        #impls

        #[doc(#updater_doc)]
        #[derive(Clone, Debug)]
        pub struct #name(moxie_dom::prelude::Key<#state_ty>);

        impl From<moxie_dom::prelude::Key<#state_ty>> for #name {
            fn from(key: moxie_dom::prelude::Key<#state_ty>) -> Self {
                #name(key)
            }
        }

        impl #name { #(#updater_items)* }
    )
    .into()
}

/// Create an updater for a method if it's a self-mutating one.
fn make_updater_method(state_method: &syn::ImplItemMethod) -> Option<syn::ImplItemMethod> {
    let mut updater = state_method.clone();
    let name = updater.sig.ident.clone();

    // remove mutability on the receiver, if it exists, else return early
    if let Some(syn::FnArg::Receiver(recv)) = updater.sig.inputs.first_mut() {
        recv.mutability = None;
    } else {
        return None;
    }

    let arg_patterns = updater.sig.inputs.iter().skip(1).map(|arg| match arg {
        syn::FnArg::Receiver(_) => unreachable!("skipped the receiver"),
        syn::FnArg::Typed(arg) => arg.pat.clone(),
    });

    updater.block = syn::parse_quote!({
        self.0.mutate(|this| this.#name( #(#arg_patterns),* ));
    });

    Some(updater)
}
