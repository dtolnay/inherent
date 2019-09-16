use crate::default_methods;
use crate::parse::TraitImpl;
use crate::visibility::Visibility;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::token::Brace;
use syn::{Block, FnArg, Ident, ImplItem, ImplItemMethod};

pub fn inherent(vis: Visibility, mut input: TraitImpl) -> TokenStream {
    let generics = &input.generics;
    let where_clause = &input.generics.where_clause;
    let trait_ = &input.trait_;
    let ty = &input.self_ty;

    // Normal methods or other items we just pass through (and maybe generate inherent methods for)
    let mut pass_through = Vec::new();
    // List of all errors we discover when processing
    let mut errors = Vec::new();
    // We convert the default! methods into list of fake methods with empty bodies. These are
    // removed (together with all the default! macro invocations) from the output.
    let mut fake_methods = Vec::new();

    for item in input.items {
        match item {
            ImplItem::Macro(ref item) if item.mac.path.is_ident("default") => {
                match item.mac.parse_body_with(default_methods::parse) {
                    Ok(body) => fake_methods.extend(body.into_iter().map(|item| {
                        ImplItem::Method(ImplItemMethod {
                            attrs: item.attrs,
                            vis: syn::Visibility::Inherited,
                            defaultness: None,
                            sig: item.sig,
                            block: Block {
                                brace_token: Brace::default(),
                                stmts: Vec::new(),
                            },
                        })
                    })),
                    Err(e) => errors.push(e.to_compile_error()),
                }
            }
            _ => pass_through.push(item),
        }
    }
    input.items = pass_through;

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
        #(#errors)*

        impl #generics #ty #where_clause {
            #(#fwd_methods)*
        }

        #input
    }
}
