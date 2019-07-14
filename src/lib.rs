extern crate proc_macro;

mod expand;
mod parse;
mod visibility;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::parse::TraitImpl;
use crate::visibility::Visibility;

#[proc_macro_attribute]
pub fn inherent(args: TokenStream, input: TokenStream) -> TokenStream {
    let vis = parse_macro_input!(args as Visibility);
    let input = parse_macro_input!(input as TraitImpl);
    expand::inherent(vis, input).into()
}
