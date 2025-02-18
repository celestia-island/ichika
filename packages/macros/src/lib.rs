mod template;
mod tools;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use template::generate_closures;
use tools::PipeMacros;

#[proc_macro]
pub fn pipe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PipeMacros);

    let closure_impl_list = generate_closures("_Stage", input.closures.clone()).expect(
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
