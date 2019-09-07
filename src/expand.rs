use std::mem;

use crate::default_methods::DefaultBody;
use crate::parse::TraitImpl;
use crate::visibility::Visibility;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{FnArg, Ident, ImplItem};

pub fn inherent(vis: Visibility, mut input: TraitImpl) -> TokenStream {
    let generics = &input.generics;
    let where_clause = &input.generics.where_clause;
    let trait_ = &input.trait_;
    let ty = &input.self_ty;

    // Remove the default! section from the output
    let (pass_through, defaults) = mem::replace(&mut input.items, Vec::new())
        .into_iter()
        .partition(|item| match item {
            ImplItem::Macro(mac) if mac.mac.path.is_ident("default") => false,
            _ => true,
        });
    input.items = pass_through;

    // Convert the default! section(s) into fake method impls so it is properly generated. This
    // adds empty bodies to them, but it doesn't matter because we don't use them.
    let fake_methods = defaults
        .into_iter()
        .flat_map(|mac| if let ImplItem::Macro(mac) = mac {
            mac.mac.parse_body::<DefaultBody>().unwrap().0
        } else {
            unreachable!();
        })
        .collect::<Vec<_>>();

    let fwd_methods = fake_methods.iter().chain(&input.items).filter_map(|item| {
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
