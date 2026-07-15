mod template;
mod tools;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use template::{generate_closures, rewrite_names};
use tools::PipeMacros;

fn emit_compile_error(e: anyhow::Error) -> TokenStream {
    let msg = e.to_string();
    quote! { compile_error!(#msg) }.into()
}

#[proc_macro]
pub fn pipe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PipeMacros);
    let global_constraints = input.global_constraints.clone();

    let input = match rewrite_names(input) {
        Ok(v) => v,
        Err(e) => return emit_compile_error(e),
    };
    let closure_impl_list = match generate_closures(input.clone()) {
        Ok(v) => v,
        Err(e) => return emit_compile_error(e),
    };
    let pool_decl = match template::generate_pool(input, global_constraints) {
        Ok(v) => v,
        Err(e) => return emit_compile_error(e),
    };

    quote! {
        {
            #closure_impl_list

            #pool_decl

            _Pool::new()
        }
    }
    .into()
}
