use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

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
                PipeNodeFlatten::Closure(closure) => generate_closure(closure.clone())?,
                PipeNodeFlatten::Map(nodes) => {
                    // Flatten all nested PipeNodeFlatten nodes from match arms
                    let nested_steps: Vec<PipeNodeFlatten> = nodes
                        .iter()
                        .map(|node| {
                            // Only extract Closure nodes from match arms
                            // Map nodes within match arms will be handled by recursion
                            match &node.body {
                                PipeNodeFlatten::Closure(c) => PipeNodeFlatten::Closure(c.clone()),
                                PipeNodeFlatten::Map(nested) => PipeNodeFlatten::Map(nested.clone()),
                                PipeNodeFlatten::Dispatcher(_) => {
                                    // Dispatcher should not appear in old-style Map nodes
                                    unreachable!("Dispatcher in old-style Map node")
                                }
                            }
                        })
                        .collect();

                    // Recursively generate closures for all nested nodes
                    generate_closures(nested_steps)?
                }
                PipeNodeFlatten::Dispatcher(dispatcher) => generate_dispatcher(dispatcher.clone())?,
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
