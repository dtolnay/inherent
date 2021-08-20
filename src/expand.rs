use crate::parse::TraitImpl;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use syn::{FnArg, Ident, ImplItem, ImplItemMethod, Item, Path, Stmt, Visibility};

pub fn inherent(mut input: TraitImpl) -> TokenStream {
    let generics = &input.generics;
    let where_clause = &input.generics.where_clause;
    let trait_ = &input.trait_;
    let ty = &input.self_ty;

    let fwd_methods: Vec<_> = input
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Method(method) => Some(fwd_method(trait_, method)),
            _ => None,
        })
        .collect();

    input.items = input
        .items
        .into_iter()
        .filter_map(|item| match item {
            ImplItem::Method(mut method) => {
                if inherit_default_implementation(&method) {
                    None
                } else {
                    method.vis = Visibility::Inherited;
                    Some(ImplItem::Method(method))
                }
            }
            item => Some(item),
        })
        .collect();

    quote! {
        impl #generics #ty #where_clause {
            #(#fwd_methods)*
        }

        #input
    }
}

fn fwd_method(trait_: &Path, method: &ImplItemMethod) -> TokenStream {
    let attrs = &method.attrs;
    let vis = &method.vis;
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

    let has_doc = attrs.iter().any(|attr| attr.path.is_ident("doc"));
    let default_doc = if has_doc {
        None
    } else {
        let mut link = String::new();
        for segment in &trait_.segments {
            link += &segment.ident.to_string();
            link += "::";
        }
        let msg = format!("See [`{}{}`]", link, ident);
        Some(quote!(#[doc = #msg]))
    };

    quote! {
        #(#attrs)*
        #default_doc
        #vis #constness #asyncness #unsafety #abi fn #ident #generics (
            #(#arg_pat,)*
        ) #output #where_clause {
            <Self as #trait_>::#ident::<#(#types,)*>(#(#arg_val,)*)
        }
    }
}

fn inherit_default_implementation(method: &ImplItemMethod) -> bool {
    method.block.stmts.len() == 1
        && match &method.block.stmts[0] {
            Stmt::Item(Item::Verbatim(verbatim)) => {
                let mut iter = verbatim.clone().into_iter();
                match iter.next() {
                    Some(TokenTree::Punct(punct)) => {
                        punct.as_char() == ';' && iter.next().is_none()
                    }
                    _ => false,
                }
            }
            _ => false,
        }
}
