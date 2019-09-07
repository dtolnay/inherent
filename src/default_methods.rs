use syn::{Block, ImplItem, ImplItemMethod, TraitItemMethod};
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::token::Brace;

pub(crate) struct DefaultBody(pub(crate) Vec<ImplItem>);

impl Parse for DefaultBody {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            let item = input.parse::<TraitItemMethod>()?;
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
        Ok(DefaultBody(items))
    }
}

