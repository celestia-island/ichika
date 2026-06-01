use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    token, Expr, Ident, Token,
};

use super::closure::ThreadConstraints;
use super::ClosureMacros;

#[derive(Debug, Clone)]
pub enum PipeNode {
    Closure(Box<ClosureMacros>),
    Map(Vec<MatchNode>),
}

#[derive(Debug, Clone)]
pub struct MatchNode {
    pub condition: Expr,
    pub body: PipeNode,
}

#[derive(Debug, Clone)]
pub struct PipeMacros {
    pub global_constraints: Option<ThreadConstraints>,
    pub closures: Vec<PipeNode>,
}

impl Parse for PipeMacros {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Check for global constraints: (max_threads: 4, min_threads: 1)
        let global_constraints = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let mut max_threads = None;
            let mut min_threads = None;

            while !content.is_empty() {
                let key: Ident = content.parse()?;
                content.parse::<Token![:]>()?;
                let value: Expr = content.parse()?;

                match key.to_string().as_str() {
                    "max_threads" => max_threads = Some(value),
                    "min_threads" => min_threads = Some(value),
                    _ => {
                        return Err(syn::Error::new(
                            key.span(),
                            format!("Unknown constraint: {}", key),
                        ))
                    }
                }

                if content.is_empty() {
                    break;
                }
                content.parse::<Token![,]>()?;
            }

            // Expect comma after global constraints
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }

            Some(ThreadConstraints {
                max_threads,
                min_threads,
            })
        } else {
            None
        };

        let mut closures = vec![];

        fn dfs(input: ParseStream) -> syn::Result<PipeNode> {
            if input.peek(Token![|]) || input.peek(Token![async]) {
                let closure = input.parse()?;
                Ok(PipeNode::Closure(Box::new(closure)))
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
                Ok(PipeNode::Closure(Box::new(input.parse()?)))
            }
        }

        while !input.is_empty() {
            let node = dfs(input)?;
            closures.push(node);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            global_constraints,
            closures,
        })
    }
}
