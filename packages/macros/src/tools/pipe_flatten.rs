use proc_macro2::TokenStream;
use syn::{Expr, Ident, TypePath};

use super::ThreadConstraints;

#[derive(Debug, Clone)]
pub struct ClosureMacrosFlatten {
    pub id: Ident,
    pub constraints: Option<ThreadConstraints>,
    pub is_async: bool,
    pub arg: Vec<Ident>,
    pub arg_ty: Vec<TypePath>,
    pub ret_ty: TypePath,
    pub body: TokenStream,
}

#[derive(Debug, Clone)]
pub enum PipeNodeFlatten {
    Closure(ClosureMacrosFlatten),
    Dispatcher(DispatcherMacrosFlatten),
}

#[derive(Debug, Clone)]
pub struct DispatcherMacrosFlatten {
    pub id: Ident,
    pub input_ty: TypePath,
    pub output_ty: TypePath,
    pub branches: Vec<BranchInfo>,
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    #[allow(dead_code)]
    pub condition: Expr,
    pub target_id: Ident,
}
