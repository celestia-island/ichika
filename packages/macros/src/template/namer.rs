use anyhow::{anyhow, Result};
use proc_macro2::Span;
use syn::Ident;

use crate::tools::{
    pipe::{MatchNode, PipeNode},
    pipe_flatten::{BranchInfo, ClosureMacrosFlatten, DispatcherMacrosFlatten, PipeNodeFlatten},
    ClosureMacros, PipeMacros,
};

/// Rewrite a single PipeNode, potentially returning multiple nodes.
/// For Map nodes, this returns a Dispatcher followed by branch closures.
/// For Closure nodes, this returns a single Closure node.
fn rewrite_name(prefix: impl ToString, step: PipeNode) -> Result<Vec<PipeNodeFlatten>> {
    Ok(match step {
        PipeNode::Closure(closure) => {
            let id = closure
                .id
                .clone()
                .unwrap_or(Ident::new(&prefix.to_string(), Span::call_site()));
            let ClosureMacros {
                constraints,
                is_async,
                arg,
                arg_ty,
                ret_ty,
                body,
                ..
            } = closure.clone();

            vec![PipeNodeFlatten::Closure(ClosureMacrosFlatten {
                id,
                constraints,
                is_async,
                arg,
                arg_ty,
                ret_ty,
                body,
            })]
        }
        PipeNode::Map(nodes) => {
            // Generate dispatcher ID
            let dispatcher_id = Ident::new(
                &format!("{}_match_dispatcher", prefix.to_string()),
                Span::call_site(),
            );

            // Get input type from first branch - dispatcher's Response type is same as Request type
            // since it routes without transforming the value
            let first_branch = nodes.first().ok_or(anyhow!("Empty match"))?;
            let input_ty = match &first_branch.body {
                PipeNode::Closure(c) => c
                    .arg_ty
                    .first()
                    .cloned()
                    .ok_or(anyhow!("Missing input type"))?,
                _ => return Err(anyhow!("Match arm must be a closure")),
            };
            let output_ty = input_ty.clone(); // Dispatcher routes without transforming, so output type = input type

            // Process each branch
            let mut all_nodes = Vec::new();
            let mut branches = Vec::new();

            for (index, MatchNode { condition, body }) in nodes.into_iter().enumerate() {
                let branch_prefix = format!("{}_{}", prefix.to_string(), index);
                let mut branch_nodes = rewrite_name(branch_prefix, body)?;

                // The first node should be the closure for this branch
                let target_id = match branch_nodes.first() {
                    Some(PipeNodeFlatten::Closure(c)) => c.id.clone(),
                    _ => return Err(anyhow!("Branch must be a closure")),
                };

                // Add all branch nodes to our collection
                all_nodes.append(&mut branch_nodes);

                // Add branch info to dispatcher
                branches.push(BranchInfo {
                    condition,
                    target_id,
                });
            }

            // Create the dispatcher node
            let dispatcher = PipeNodeFlatten::Dispatcher(DispatcherMacrosFlatten {
                id: dispatcher_id,
                input_ty,
                output_ty,
                branches,
            });

            // Return dispatcher followed by all branch closures
            all_nodes.insert(0, dispatcher);
            all_nodes
        }
    })
}

pub(crate) fn rewrite_names(pipes: PipeMacros) -> Result<Vec<PipeNodeFlatten>> {
    let mut all_nodes = Vec::new();

    for (index, closure) in pipes.closures.iter().enumerate() {
        let prefix = if let PipeNode::Closure(ClosureMacros { id: Some(id), .. }) = closure {
            id.clone()
        } else {
            Ident::new(&format!("_step_{}", index), Span::call_site())
        };

        let mut nodes = rewrite_name(prefix, closure.clone())?;
        all_nodes.append(&mut nodes);
    }

    Ok(all_nodes)
}
