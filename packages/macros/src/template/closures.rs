use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use super::generate_closure;
use crate::tools::pipe_flatten::PipeNodeFlatten;

pub(crate) fn generate_closures(steps: Vec<PipeNodeFlatten>) -> Result<TokenStream> {
    let closure_impl_list = steps
        .iter()
        .enumerate()
        .map(|(_id, node)| {
            Ok(match node {
                PipeNodeFlatten::Closure(closure) => generate_closure(closure.clone())?,
                PipeNodeFlatten::Map(nodes) => {
                    // Flatten all nested PipeNodeFlatten nodes from match arms
                    let nested_steps: Vec<PipeNodeFlatten> = nodes
                        .iter()
                        .filter_map(|node| {
                            // Only extract Closure nodes from match arms
                            // Map nodes within match arms will be handled by recursion
                            match &node.body {
                                PipeNodeFlatten::Closure(c) => Some(PipeNodeFlatten::Closure(c.clone())),
                                PipeNodeFlatten::Map(nested) => Some(PipeNodeFlatten::Map(nested.clone())),
                            }
                        })
                        .collect();

                    // Recursively generate closures for all nested nodes
                    generate_closures(nested_steps)?
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
