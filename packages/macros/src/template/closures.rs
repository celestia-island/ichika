use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use super::generate_closure;
use crate::tools::pipe_flatten::PipeNodeFlatten;

pub(crate) fn generate_closures(stages: Vec<PipeNodeFlatten>) -> Result<TokenStream> {
    let closure_impl_list = stages
        .iter()
        .enumerate()
        .map(|(id, node)| {
            Ok(match node {
                PipeNodeFlatten::Closure(closure) => generate_closure(closure.clone())?,
                PipeNodeFlatten::Map(nodes) => {
                    todo!("递归调用 generate_closures 函数，生成多个闭包的实现")
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
