use anyhow::Result;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use super::generate_closure;
use crate::tools::{pipe::PipeNode, ClosureMacros};

pub(crate) fn generate_closures(
    prefix: impl ToString,
    stages: Vec<PipeNode>,
) -> Result<TokenStream> {
    let closure_impl_list = stages
        .iter()
        .enumerate()
        .map(|(id, node)| {
            Ok(match node {
                PipeNode::Closure(closure) => generate_closure(ClosureMacros {
                    id: closure.id.clone().or_else(|| {
                        Some(Ident::new(
                            &format!("{}_{}", prefix.to_string(), id),
                            Span::call_site(),
                        ))
                    }),
                    ..closure.clone()
                })?,
                PipeNode::Map(nodes) => {
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
