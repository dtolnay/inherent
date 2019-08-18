use crate::parse::TraitImpl;
use crate::visibility::Visibility;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{FnArg, Ident, ImplItem};

pub fn inherent(vis: Visibility, input: TraitImpl) -> TokenStream {
    let generics = &input.generics;
    let where_clause = &input.generics.where_clause;
    let trait_ = &input.trait_;
    let ty = &input.self_ty;

    let fwd_methods = input.items.iter().filter_map(|item| {
        let method = match item {
            ImplItem::Method(method) => method,
            _ => return None,
        };

        let attrs = &method.attrs;
        let constness = &method.sig.constness;
        let asyncness = &method.sig.asyncness;
        let unsafety = &method.sig.unsafety;
        let abi = &method.sig.abi;
        let ident = &method.sig.ident;
        let generics = &method.sig.generics;
        let output = &method.sig.output;
        let where_clause = &method.sig.generics.where_clause;

        let (arg_pat, arg_val): (Vec<_>, Vec<_>) = method
            .sig
            .inputs
            .iter()
            .enumerate()
            .map(|(i, input)| match input {
                FnArg::Receiver(receiver) => {
                    if receiver.reference.is_some() {
                        (quote!(#receiver), quote!(self))
                    } else {
                        (quote!(self), quote!(self))
                    }
                }
                FnArg::Typed(arg) => {
                    let var = Ident::new(&format!("__arg{}", i), Span::call_site());
                    let ty = &arg.ty;
                    (quote!(#var: #ty), quote!(#var))
                }
            })
            .unzip();

        let types = generics.type_params().map(|param| &param.ident);

        Some(quote! {
            #(#attrs)*
            #vis #constness #asyncness #unsafety #abi fn #ident #generics (
                #(#arg_pat,)*
            ) #output #where_clause {
                <Self as #trait_>::#ident::<#(#types,)*>(#(#arg_val,)*)
            }
        })
    });

    quote! {
        impl #generics #ty #where_clause {
            #(#fwd_methods)*
        }

        #input
    }
}
