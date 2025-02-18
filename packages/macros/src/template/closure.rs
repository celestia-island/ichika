use anyhow::{anyhow, Result};
use proc_macro2::TokenStream;
use quote::quote;

use crate::tools::ClosureMacros;

pub(crate) fn generate_closure(closure: ClosureMacros) -> Result<TokenStream> {
    let id = closure
        .id
        .ok_or(anyhow!("Closure must have an identifier"))?;
    let id_raw = quote! { stringify!(#id) };
    let ClosureMacros {
        arg_ty,
        ret_ty,
        body,
        ..
    } = closure;

    // TODO: Support async closures
    Ok(quote! {
      struct #id;

      impl ::ichika::node::ThreadNode for #id {
        type Request = #arg_ty;
        type Response = #ret_ty;

        fn run(&self, request: Self::Request) -> Self::Response {
          #body
        }
      }

      impl ::ichika::node::ThreadNodeEnum for #id {
        fn id() -> &'static str {
          #id_raw
        }
      }
    })
}
