use proc_macro2::TokenStream;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token, TypePath,
};

#[derive(Debug, Clone)]
pub struct ClosureMacros {
    pub id: Option<Ident>, // TODO: Not done yet
    pub is_async: bool,
    pub arg: Ident,
    pub arg_ty: TypePath,
    pub ret_ty: TypePath,
    pub body: TokenStream,
}

impl Parse for ClosureMacros {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // async |ident: Ty| -> Ty { ... }
        let is_async = {
            if input.peek(Token![async]) {
                input.parse::<Token![async]>()?;
                true
            } else {
                false
            }
        };

        input.parse::<Token![|]>()?;
        let arg = input.parse()?;
        input.parse::<Token![:]>()?;
        let arg_ty = input.parse()?;
        input.parse::<Token![|]>()?;

        input.parse::<Token![->]>()?;
        let ret_ty = input.parse()?;

        let content;
        braced!(content in input);
        let body = content.parse()?;

        Ok(Self {
            id: None,
            is_async,
            arg,
            arg_ty,
            ret_ty,
            body,
        })
    }
}
