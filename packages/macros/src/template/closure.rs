use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use crate::tools::pipe_flatten::ClosureMacrosFlatten;

pub(crate) fn generate_closure(closure: ClosureMacrosFlatten) -> Result<TokenStream> {
    let ClosureMacrosFlatten {
        id,
        arg,
        arg_ty,
        ret_ty,
        body,
        ..
    } = closure;
    let id_raw = quote! { stringify!(#id) };

    // TODO: Support async closures
    Ok(quote! {
      struct #id;

      impl ::ichika::node::ThreadNode for #id {
        type Request = #arg_ty;
        type Response = #ret_ty;

        fn run(#arg: Self::Request) -> ::ichika::Status<Self::Response, ::ichika::anyhow::Error> {
          { #body }.into_status()
        }
      }

      impl ::ichika::node::ThreadNodeEnum for #id {
        fn id() -> &'static str {
          #id_raw
        }
      }
    })
}
