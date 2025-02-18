mod template;
mod tools;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use template::{generate_closures, rewrite_names};
use tools::PipeMacros;

#[proc_macro]
pub fn pipe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PipeMacros);
    let input = rewrite_names(input)
        .expect("Failed to rewrite names. Please check the error message above.");

    let closure_impl_list = generate_closures(input.clone()).expect(
        "Failed to generate closure implementation list. Please check the error message above.",
    );

    // TODO: Not done yet
    quote! {
        {
            #closure_impl_list

            vec!["todo"]
        }
    }
    .into()
}
