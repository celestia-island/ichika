use proc_macro2::TokenStream;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Token, TypePath,
};

#[derive(Debug, Clone)]
pub struct ClosureMacros {
    pub id: Option<Ident>,
    // TODO: Allow set limitation after id
    //       like `xxx(max_threads_count: 1, max_tasks_count: 1) |ident: Ty| -> Ty { ... }`
    pub is_async: bool,
    pub arg: Ident,
    pub arg_ty: TypePath,
    pub ret_ty: TypePath,
    pub body: TokenStream,
}

impl Parse for ClosureMacros {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // async |ident: Ty| -> Ty { ... }
        let id = {
            if input.peek(Ident) {
                let id = input.parse()?;
                input.parse::<Token![:]>()?;
                Some(id)
            } else {
                None
            }
        };

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
            id,
            is_async,
            arg,
            arg_ty,
            ret_ty,
            body,
        })
    }
}
