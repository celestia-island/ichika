use syn::{
    braced,
    parse::{Parse, ParseStream},
    Expr, Token,
};

use super::ClosureMacros;

#[derive(Debug, Clone)]
pub enum PipeNode {
    Closure(ClosureMacros),
    Map(Vec<MatchNode>),
}

#[derive(Debug, Clone)]
pub struct MatchNode {
    condition: Expr,
    body: PipeNode,
}

#[derive(Debug, Clone)]
pub struct PipeMacros {
    // TODO: Allow set limitation before closures
    //       like `pipe![
    //              (max_threads_count: 1, max_tasks_count: 1),
    //              |ident: Ty| -> Ty { ... },
    //              ...
    //            ]`
    closures: Vec<PipeNode>,
}

impl Parse for PipeMacros {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut closures = vec![];

        fn dfs(input: ParseStream) -> syn::Result<PipeNode> {
            if input.peek(Token![|]) || input.peek(Token![async]) {
                let closure = input.parse()?;
                Ok(PipeNode::Closure(closure))
            } else if input.peek(Token![match]) {
                input.parse::<Token![match]>()?;
                let content;
                braced!(content in input);
                let mut nodes = vec![];

                while !content.is_empty() {
                    let condition = content.parse()?;
                    content.parse::<Token![=>]>()?;
                    let body = dfs(&content)?;
                    nodes.push(MatchNode { condition, body });

                    if content.peek(Token![,]) {
                        content.parse::<Token![,]>()?;
                    }
                }

                Ok(PipeNode::Map(nodes))
            } else {
                Ok(PipeNode::Closure(input.parse()?))
            }
        }

        while !input.is_empty() {
            let node = dfs(input)?;
            closures.push(node);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { closures })
    }
}
