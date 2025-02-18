use anyhow::Result;
use proc_macro2::Span;
use syn::Ident;

use crate::tools::{
    pipe::{MatchNode, PipeNode},
    pipe_flatten::{ClosureMacrosFlatten, MatchNodeFlatten, PipeNodeFlatten},
    ClosureMacros, PipeMacros,
};

fn rewrite_name(prefix: impl ToString, stage: PipeNode) -> Result<PipeNodeFlatten> {
    Ok(match stage {
        PipeNode::Closure(closure) => {
            let id = closure
                .id
                .clone()
                .unwrap_or(Ident::new(&prefix.to_string(), Span::call_site()));
            let ClosureMacros {
                is_async,
                arg,
                arg_ty,
                ret_ty,
                body,
                ..
            } = closure.clone();

            PipeNodeFlatten::Closure(ClosureMacrosFlatten {
                id,
                is_async,
                arg,
                arg_ty,
                ret_ty,
                body,
            })
        }
        PipeNode::Map(nodes) => {
            let nodes = nodes
                .into_iter()
                .enumerate()
                .map(|(index, MatchNode { condition, body })| {
                    rewrite_name(format!("{}_{}", prefix.to_string(), index), body.clone())
                        .map(|body| MatchNodeFlatten { condition, body })
                })
                .collect::<Result<Vec<_>>>()?;
            PipeNodeFlatten::Map(nodes)
        }
    })
}

pub(crate) fn rewrite_names(pipes: PipeMacros) -> Result<Vec<PipeNodeFlatten>> {
    let closures = pipes
        .closures
        .iter()
        .enumerate()
        .map(|(index, closure)| {
            rewrite_name(
                if let PipeNode::Closure(ClosureMacros { id: Some(id), .. }) = closure {
                    id.clone()
                } else {
                    Ident::new(&format!("_Stage_{}", index), Span::call_site())
                },
                closure.clone(),
            )
        })
        .collect::<Vec<_>>();
    let closures = closures.into_iter().collect::<Result<Vec<_>>>()?;

    Ok(closures)
}
