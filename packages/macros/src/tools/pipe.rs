use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    token, Expr, Ident, Token,
};

#[derive(Debug, Clone)]
pub struct PipeMacros {}

impl Parse for PipeMacros {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {})
    }
}
