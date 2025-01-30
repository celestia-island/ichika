use syn::{
    parse::{Parse, ParseStream},
    Token,
};

use super::ClosureMacros;

#[derive(Debug, Clone)]
pub struct PipeMacros {
    closures: Vec<ClosureMacros>,
}

impl Parse for PipeMacros {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut closures = vec![];
        while !input.is_empty() {
            let closure = input.parse()?;
            closures.push(closure);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { closures })
    }
}
