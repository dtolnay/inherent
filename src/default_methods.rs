use syn::{Block, ImplItem, ImplItemMethod, TraitItemMethod};
use syn::parse::{Error, Parse, ParseStream, Result as ParseResult};
use syn::token::Brace;

pub(crate) struct DefaultBody(pub(crate) Vec<ImplItem>);

impl Parse for DefaultBody {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut items = Vec::new();
        let mut error = None;
        while !input.is_empty() {
            let item = input.parse::<TraitItemMethod>()?;
            if let Some(body) = item.default {
                let new_err = Error::new_spanned(body, "Default method can't have a body");
                match &mut error {
                    None => error = Some(new_err),
                    Some(e) => e.combine(new_err),
                }
            } else {
                items.push(ImplItem::Method(ImplItemMethod {
                    attrs: item.attrs,
                    vis: syn::Visibility::Inherited,
                    defaultness: None,
                    sig: item.sig,
                    block: Block {
                        brace_token: Brace::default(),
                        stmts: Vec::new(),
                    },
                }));
            }
        }
        match error {
            None => Ok(DefaultBody(items)),
            Some(err) => Err(err),
        }
    }
}

