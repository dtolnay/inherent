//! ##### An attribute macro to make trait methods callable without the trait in scope.
//!
//! # Example
//!
//! ```rust
//! mod types {
//!     use inherent::inherent;
//!
//!     trait Trait {
//!         fn f(self);
//!     }
//!
//!     pub struct Struct;
//!
//!     #[inherent(pub)]
//!     impl Trait for Struct {
//!         fn f(self) {}
//!     }
//! }
//!
//! fn main() {
//!     // types::Trait is not in scope, but method can be called.
//!     types::Struct.f();
//! }
//! ```
//!
//! Without the `inherent` macro on the trait impl, this would have failed with the
//! following error:
//!
//! ```console
//! error[E0599]: no method named `f` found for type `types::Struct` in the current scope
//!   --> src/main.rs:18:19
//!    |
//! 8  |     pub struct Struct;
//!    |     ------------------ method `f` not found for this
//! ...
//! 18 |     types::Struct.f();
//!    |                   ^
//!    |
//!    = help: items from traits can only be used if the trait is implemented and in scope
//!    = note: the following trait defines an item `f`, perhaps you need to implement it:
//!            candidate #1: `types::Trait`
//! ```
//!
//! The `inherent` macro expands to inherent methods on the `Self` type of the trait
//! impl that forward to the trait methods. In the case above, the generated code
//! would be:
//!
//! ```rust
//! impl Struct {
//!     pub fn f(self) {
//!         <Self as Trait>::f(self)
//!     }
//! }
//! ```
//!
//! # Visibility
//!
//! Ordinary trait methods have the same visibility as the trait or the `Self` type,
//! whichever's is smaller. Neither of these visibilities is knowable to the
//! `inherent` macro, so we simply emit all inherent methods as private by default.
//! A different visibility may be specified explicitly in the `inherent` macro
//! invocation.
//!
//! ```rust
//! #[inherent]  // private inherent methods are the default
//!
//! #[inherent(pub)]  // all methods pub
//!
//! #[inherent(crate)]  // all methods pub(crate)
//!
//! #[inherent(in path::to)]  // all methods pub(in path::to)
//! ```

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
