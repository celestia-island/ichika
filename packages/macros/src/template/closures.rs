use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use super::generate_closure;
use crate::tools::pipe_flatten::{DispatcherMacrosFlatten, PipeNodeFlatten};

pub(crate) fn generate_dispatcher(dispatcher: DispatcherMacrosFlatten) -> Result<TokenStream> {
    let DispatcherMacrosFlatten {
        id,
        input_ty,
        output_ty,
        branches,
    } = dispatcher;
    let id_raw = quote! { stringify!(#id) };

    // Generate match arms for the dispatcher
    let match_arms = branches.iter().map(|branch| {
        let condition = &branch.condition;
        let target_id = &branch.target_id;
        let target_str = target_id.to_string();
        quote! {
            #condition => ::ichika::Status::Switch((#target_str, req)),
        }
    });

    // Dispatcher routes without transforming, so input_ty == output_ty
    // The returned Status<Request, Error> matches the trait's Status<Response, Error>
    Ok(quote! {
        #[allow(non_camel_case_types)]
        struct #id;

        impl ::ichika::node::ThreadNode for #id {
            type Request = #input_ty;
            type Response = #output_ty;

            fn run(req: Self::Request) -> ::ichika::Status<Self::Response, ::ichika::anyhow::Error> {
                match req {
                    #( #match_arms )*
                }
            }
        }

        impl ::ichika::node::ThreadNodeEnum for #id {
            fn id() -> &'static str {
                #id_raw
            }
        }
    })
}

pub(crate) fn generate_closures(steps: Vec<PipeNodeFlatten>) -> Result<TokenStream> {
    let closure_impl_list = steps
        .iter()
        .map(|node| {
            Ok(match node {
                PipeNodeFlatten::Closure(closure) => generate_closure(*closure.clone())?,
                PipeNodeFlatten::Dispatcher(dispatcher) => {
                    generate_dispatcher(*dispatcher.clone())?
                }
            })
        })
        .collect::<Vec<Result<_>>>();
    let closure_impl_list = closure_impl_list
        .into_iter()
        .collect::<Result<Vec<_>>>()
        .expect(
            "Failed to generate closure implementation list. Please check the error message above.",
        );

    Ok(quote! {
      #(#closure_impl_list)*
    })
}
