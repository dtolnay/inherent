use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::mem;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::token::Brace;
use syn::{Attribute, Generics, ImplItem, ImplModifiers, ItemImpl, Path, Token, Type};

#[derive(Clone)]
pub struct TraitImpl {
    pub attrs: Vec<Attribute>,
    pub defaultness: Option<Token![default]>,
    pub unsafety: Option<Token![unsafe]>,
    pub impl_token: Token![impl],
    pub generics: Generics,
    pub polarity: Option<Token![!]>,
    pub trait_: Path,
    pub for_token: Token![for],
    pub self_ty: Type,
    pub brace_token: Brace,
    pub items: Vec<ImplItem>,
}

impl Parse for TraitImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut imp: ItemImpl = input.parse()?;

        let Some((trait_, for_token)) = imp.trait_ else {
            return Err(Error::new(
                Span::call_site(),
                "must be placed on a trait impl",
            ));
        };

        let defaultness = mem::take(&mut imp.modifiers.defaultness);
        let polarity = mem::take(&mut imp.modifiers.polarity);
        imp.modifiers.require_empty()?;

        Ok(TraitImpl {
            attrs: imp.attrs,
            defaultness,
            unsafety: imp.unsafety,
            impl_token: imp.impl_token,
            generics: imp.generics,
            polarity,
            trait_,
            for_token,
            self_ty: *imp.self_ty,
            brace_token: imp.brace_token,
            items: imp.items,
        })
    }
}

impl ToTokens for TraitImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let imp = self.clone();

        ItemImpl::to_tokens(
            &ItemImpl {
                attrs: imp.attrs,
                modifiers: {
                    let mut modifiers = ImplModifiers::default();
                    modifiers.defaultness = imp.defaultness;
                    modifiers.polarity = imp.polarity;
                    modifiers
                },
                unsafety: imp.unsafety,
                impl_token: imp.impl_token,
                generics: imp.generics,
                trait_: Some((imp.trait_, imp.for_token)),
                self_ty: Box::new(imp.self_ty),
                brace_token: imp.brace_token,
                items: imp.items,
            },
            tokens,
        );
    }
}
