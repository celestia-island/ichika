use proc_macro2::TokenStream;
use syn::{Expr, Ident, TypePath};

#[derive(Debug, Clone)]
pub struct ClosureMacrosFlatten {
    pub id: Ident,
    pub is_async: bool,
    pub arg: Vec<Ident>,
    pub arg_ty: Vec<TypePath>,
    pub ret_ty: TypePath,
    pub body: TokenStream,
}

#[derive(Debug, Clone)]
pub enum PipeNodeFlatten {
    Closure(ClosureMacrosFlatten),
    Map(Vec<MatchNodeFlatten>),
}

#[derive(Debug, Clone)]
pub struct MatchNodeFlatten {
    pub condition: Expr,
    pub body: PipeNodeFlatten,
}
