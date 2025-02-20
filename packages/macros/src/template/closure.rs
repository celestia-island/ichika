use anyhow::{anyhow, Result};
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

    if closure.is_async {
        if cfg!(feature = "tokio") {
            Ok(quote! {
              struct #id;

              impl ::ichika::node::ThreadNode for #id {
                type Request = #arg_ty;
                type Response = #ret_ty;

                fn run(#arg: Self::Request) -> ::ichika::Status<Self::Response, ::ichika::anyhow::Error> {
                  let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                  rt.block_on(async move { { #body }.await.into_status() })
                }
              }

              impl ::ichika::node::ThreadNodeEnum for #id {
                fn id() -> &'static str {
                  #id_raw
                }
              }
            })
        } else if cfg!(feature = "async-std") {
            Ok(quote! {
              struct #id;

              impl ::ichika::node::ThreadNode for #id {
                type Request = #arg_ty;
                type Response = #ret_ty;

                fn run(#arg: Self::Request) -> ::ichika::Status<Self::Response, ::ichika::anyhow::Error> {
                  async_std::task::block_on(async move { { #body }.await.into_status() })
                }
              }

              impl ::ichika::node::ThreadNodeEnum for #id {
                fn id() -> &'static str {
                  #id_raw
                }
              }
            })
        } else {
            Err(anyhow!("No async runtime"))
        }
    } else {
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
}
