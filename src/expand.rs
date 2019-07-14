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
        let generics = &method.sig.decl.generics;
        let output = &method.sig.decl.output;
        let where_clause = &method.sig.decl.generics.where_clause;

        let (arg_pat, arg_val): (Vec<_>, Vec<_>) = method
            .sig
            .decl
            .inputs
            .iter()
            .enumerate()
            .map(|(i, input)| match input {
                FnArg::SelfRef(arg) => (quote!(#arg), quote!(self)),
                FnArg::SelfValue(_) => (quote!(self), quote!(self)),
                FnArg::Captured(arg) => {
                    let var = Ident::new(&format!("__arg{}", i), Span::call_site());
                    let ty = &arg.ty;
                    (quote!(#var: #ty), quote!(#var))
                }
                FnArg::Inferred(_) | FnArg::Ignored(_) => unimplemented!(),
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
