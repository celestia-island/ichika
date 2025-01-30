mod template;
mod tools;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use tools::PipeMacros;

#[proc_macro]
pub fn pipe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PipeMacros);

    // TODO: Not done yet
    quote! {
        vec!["todo"]
    }
    .into()
}
