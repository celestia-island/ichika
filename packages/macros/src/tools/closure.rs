use proc_macro2::TokenStream;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    token, Expr, Ident, Token, TypePath,
};

#[derive(Debug, Clone)]
pub struct ThreadConstraints {
    pub max_threads: Option<Expr>,
    pub min_threads: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ClosureMacros {
    pub id: Option<Ident>,
    pub constraints: Option<ThreadConstraints>,
    pub is_async: bool,
    pub arg: Vec<Ident>,
    pub arg_ty: Vec<TypePath>,
    pub ret_ty: TypePath,
    pub body: TokenStream,
}

impl Parse for ClosureMacros {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // async |ident: Ty| -> Ty { ... }
        // or with constraints: id(max_threads: 2, min_threads: 1) |ident: Ty| -> Ty { ... }
        let id = {
            if input.peek(Ident) {
                let id = input.parse()?;
                input.parse::<Token![:]>()?;
                Some(id)
            } else {
                None
            }
        };

        // Parse optional thread constraints: (max_threads: 2, min_threads: 1)
        let constraints = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let mut max_threads = None;
            let mut min_threads = None;

            while !content.is_empty() {
                let key: Ident = content.parse()?;
                content.parse::<Token![:]>()?;
                let value: Expr = content.parse()?;

                match key.to_string().as_str() {
                    "max_threads" => max_threads = Some(value),
                    "min_threads" => min_threads = Some(value),
                    _ => {
                        return Err(syn::Error::new(
                            key.span(),
                            format!("Unknown constraint: {}", key),
                        ))
                    }
                }

                if content.is_empty() {
                    break;
                }
                content.parse::<Token![,]>()?;
            }

            Some(ThreadConstraints {
                max_threads,
                min_threads,
            })
        } else {
            None
        };

        let is_async = {
            if input.peek(Token![async]) {
                input.parse::<Token![async]>()?;
                true
            } else {
                false
            }
        };

        let mut arg = vec![];
        let mut arg_ty = vec![];
        input.parse::<Token![|]>()?;
        if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);

            while !content.is_empty() {
                arg.push(content.parse()?);
                content.parse::<Token![:]>()?;
                arg_ty.push(content.parse()?);

                if content.is_empty() {
                    break;
                }

                content.parse::<Token![,]>()?;
            }
        } else {
            arg.push(input.parse()?);
            input.parse::<Token![:]>()?;
            arg_ty.push(input.parse()?);
        }
        input.parse::<Token![|]>()?;

        input.parse::<Token![->]>()?;
        let ret_ty = input.parse()?;

        let content;
        braced!(content in input);
        let body = content.parse()?;

        Ok(Self {
            id,
            constraints,
            is_async,
            arg,
            arg_ty,
            ret_ty,
            body,
        })
    }
}
