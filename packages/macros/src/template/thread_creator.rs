use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, TypePath, Expr};

use crate::tools::ThreadConstraints;

/// Generate the routing table code that maps closure names to their tx channels.
/// Only includes targets whose Request type matches the current step's Response type.
pub(crate) fn generate_routing_table(
    current_output_type: &TypePath,
    closure_names: &[Ident],
    closure_input_types: &[TypePath],
) -> TokenStream {
    let entries = closure_names.iter().zip(closure_input_types.iter()).filter_map(|(name, input_ty)| {
        // Check if input type matches current output type
        if type_matches(input_ty, current_output_type) {
            let tx_ident = format_ident!("tx_{}", name);
            let name_str = name.to_string();
            Some(quote! {
                routing_table.insert(#name_str, #tx_ident.clone());
            })
        } else {
            None
        }
    });

    quote! {
        let mut routing_table: ::std::collections::HashMap<&'static str, ::ichika::flume::Sender<#current_output_type>> = ::std::collections::HashMap::new();
        #( #entries )*
    }
}

/// Simple type matching check for routing table generation.
/// Returns true if the types appear to be the same (simple check).
pub(crate) fn type_matches(a: &TypePath, b: &TypePath) -> bool {
    // Compare the path segments of both types
    // This handles simple types like String, usize, as well as pathed types
    a.path.segments.iter().map(|seg| seg.ident.to_string()).collect::<Vec<_>>()
        == b.path.segments.iter().map(|seg| seg.ident.to_string()).collect::<Vec<_>>()
}

pub(crate) fn generate_thread_creator(
    rx_request: Ident,
    tx_response: Ident,
    target_step_ident: Ident,
    target_step_pods_ident: Ident,
    output_type: TypePath,
    routing_targets: Option<TokenStream>,
    step_constraints: Option<&ThreadConstraints>,
) -> Result<TokenStream> {
    // 这里的当前线程数量，默认情况下为根据该阶段任务及其后续所有任务的总数来决定
    // 例如，如果有三个阶段的当前任务数量 a b c，
    // 那么第一个任务的数量为 a + b + c，第二个任务的数量为 b + c，第三个任务的数量为 c
    // 这么做是为了保证每个阶段的任务数量是平均的，不会因为前一阶段的任务数量过多而导致后一阶段的任务数量过少
    // 后续开放的自定义数量的接口，设定的数值除了总线程数量和每个阶段的固定线程数外
    // 还可以设定为每个阶段不低于/不高于若干个线程
    // （会配合这里的 len() 累加，例如限制每一阶段至少预留两个，那么第一阶段就是 a + b + c + 2 * 2）

    let routing_setup = routing_targets.unwrap_or_else(|| quote! {
        let routing_table: ::std::collections::HashMap<&str, ::ichika::flume::Sender<#output_type>> = ::std::collections::HashMap::new();
    });

    // Generate thread limit check based on constraints
    let thread_limit_check = if let Some(constraints) = step_constraints {
        match (&constraints.max_threads, &constraints.min_threads) {
            (Some(max), Some(min)) => {
                quote! {
                    let max_limit = #max;
                    let min_limit = #min;
                    !#rx_request.is_empty()
                        && prev_pods_size + #target_step_pods_ident.len() < max_thread_count.min(max_limit)
                        && #target_step_pods_ident.len() < max_limit
                }
            }
            (Some(max), None) => {
                quote! {
                    let max_limit = #max;
                    !#rx_request.is_empty()
                        && prev_pods_size + #target_step_pods_ident.len() < max_thread_count.min(max_limit)
                        && #target_step_pods_ident.len() < max_limit
                }
            }
            (None, Some(min)) => {
                quote! {
                    let min_limit = #min;
                    !#rx_request.is_empty()
                        && prev_pods_size + #target_step_pods_ident.len() < max_thread_count
                        && (#target_step_pods_ident.len() < min_limit || #rx_request.len() > #target_step_pods_ident.len() * 2)
                }
            }
            (None, None) => {
                quote! {
                    !#rx_request.is_empty()
                        && prev_pods_size + #target_step_pods_ident.len() < max_thread_count
                }
            }
        }
    } else {
        quote! {
            !#rx_request.is_empty()
                && prev_pods_size + #target_step_pods_ident.len() < max_thread_count
        }
    };

    Ok(quote! {
        if #thread_limit_check
        {
            let thread = std::thread::spawn({
                let rx_request = #rx_request.clone();
                let tx_response = #tx_response.clone();
                #routing_setup

                move || {
                    while let Ok(mut req) = rx_request.try_recv() {
                        let mut attempt: usize = 0;
                        loop {
                            let res = #target_step_ident::run(req);
                            match res {
                                ::ichika::Status::Next(res) => {
                                    tx_response.send(res).unwrap();
                                    break;
                                }
                                ::ichika::Status::Exit => {
                                    break;
                                }
                                ::ichika::Status::Retry => {
                                    // Legacy retry: continue to next request
                                    break;
                                }
                                ::ichika::Status::RetryWith(policy, current_attempt, retry_req) => {
                                    if current_attempt < policy.max_attempts {
                                        std::thread::sleep(std::time::Duration::from_millis(policy.delay_ms));
                                        req = retry_req;
                                        attempt = current_attempt + 1;
                                        continue;
                                    } else {
                                        // Max attempts reached, send as-is
                                        tx_response.send(retry_req).unwrap();
                                        break;
                                    }
                                }
                                ::ichika::Status::Panic(err) => {
                                    eprintln!("Step {} panicked: {:?}", #target_step_ident::id(), err);
                                    break;
                                }
                                ::ichika::Status::Switch((target, payload)) => {
                                    if let Some(tx) = routing_table.get(target) {
                                        tx.send(payload).unwrap();
                                    } else {
                                        eprintln!("Warning: Switch target '{}' not found or type incompatible, falling back to next step", target);
                                        tx_response.send(payload).unwrap();
                                    }
                                    break;
                                }
                                ::ichika::Status::PanicSwitch((target, err)) => {
                                    eprintln!("PanicSwitch to target '{}' with error: {:?}", target, err);
                                    break;
                                }
                                ::ichika::Status::Back((target, payload)) => {
                                    if let Some(tx) = routing_table.get(target) {
                                        tx.send(payload).unwrap();
                                    } else {
                                        eprintln!("Warning: Back target '{}' not found or type incompatible, falling back to next step", target);
                                        tx_response.send(payload).unwrap();
                                    }
                                    break;
                                }
                            };
                        }
                    }
                    ::ichika::anyhow::Ok(())
                }
            });
            #target_step_pods_ident.push(ThreadPod::new(#target_step_ident::id(), thread));
        }

        let prev_pods_size = prev_pods_size + #target_step_pods_ident.len();
    })
}
