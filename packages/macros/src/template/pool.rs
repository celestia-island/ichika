use anyhow::{anyhow, Result};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::tools::{pipe_flatten::PipeNodeFlatten, ThreadConstraints};

use super::{generate_routing_table, generate_thread_creator, type_matches};

pub(crate) fn generate_pool(closures: Vec<PipeNodeFlatten>, global_constraints: Option<ThreadConstraints>) -> Result<TokenStream> {
    // Filter out Map nodes - they are routing constructs, not actual processing steps
    let closure_steps: Vec<_> = closures
        .iter()
        .filter_map(|step| match step {
            PipeNodeFlatten::Closure(c) => Some(c),
            PipeNodeFlatten::Map(_) => None, // Skip Map nodes
        })
        .collect();

    let tx_rx_request_flume_unbounded = closure_steps
        .iter()
        .map(|closure| {
            let name = closure.id.clone();
            let input_ty = closure.arg_ty.first().cloned().unwrap();

            let tx_ident = Ident::new(&format!("tx_{}", name), Span::call_site());
            let rx_ident = Ident::new(&format!("rx_{}", name), Span::call_site());
            quote! {
               let (#tx_ident, #rx_ident) = ::ichika::flume::unbounded::<#input_ty>();
            }
        })
        .collect::<Vec<TokenStream>>();
    let flume_request_unbounded_first_ident = match closure_steps.first().ok_or(anyhow!("No closure"))? {
        closure => Ident::new(&format!("tx_{}", closure.id), Span::call_site()),
    };

    let pods_name = closure_steps
        .iter()
        .map(|closure| Ident::new(&format!("pods_{}", closure.id), Span::call_site()))
        .collect::<Vec<Ident>>();
    let pods_init_let = pods_name
        .iter()
        .map(|name| {
            quote! {
                let mut #name = Vec::new();
            }
        })
        .collect::<Vec<TokenStream>>();
    let pods_init_let = quote! {
        #( #pods_init_let )*
    };

    let clean_pods_code = pods_name
        .iter()
        .map(|name| {
            quote! {
                #name.retain(|pod: &ThreadPod| pod.is_alive());
            }
        })
        .collect::<Vec<TokenStream>>();
    let has_all_pods_stop_code = pods_name
        .iter()
        .map(|name| {
            quote! {
                #name.is_empty()
            }
        })
        .collect::<Vec<TokenStream>>();
    let calculate_pods_len_code = pods_name
        .iter()
        .map(|name| {
            quote! {
                #name.len()
            }
        })
        .collect::<Vec<TokenStream>>();

    let thread_creators = closure_steps
        .iter()
        .enumerate()
        .filter_map(|(i, closure)| {
            // Find the next closure step whose input type matches current closure's output type
            let current_output_type = &closure.ret_ty;
            let next_tx = closures
                .iter()
                .skip(i + 1)
                .filter_map(|step| match step {
                    PipeNodeFlatten::Closure(c) => {
                        // Check if the next closure's input type matches current closure's output type
                        let next_input_type = c.arg_ty.first()?;
                        if type_matches(next_input_type, current_output_type) {
                            Some(Ident::new(&format!("tx_{}", c.id), Span::call_site()))
                        } else {
                            None
                        }
                    }
                    PipeNodeFlatten::Map(_) => None,
                })
                .next()
                .unwrap_or_else(|| Ident::new("tx_pods_response", Span::call_site()));

            // Collect all closure names and their input types for routing table
            let closure_names: Vec<Ident> = closure_steps.iter().map(|c| c.id.clone()).collect();
            let closure_input_types: Vec<_> = closure_steps.iter().map(|c| c.arg_ty.first().cloned().unwrap()).collect();

            // Create routing table for this thread (only type-compatible targets)
            let output_type = closure.ret_ty.clone();
            let routing_targets = Some(generate_routing_table(&output_type, &closure_names, &closure_input_types));

            // Get step constraints, falling back to global constraints
            let step_constraints = closure.constraints.as_ref().or(global_constraints.as_ref());

            Some(generate_thread_creator(
                Ident::new(&format!("rx_{}", closure.id), Span::call_site()),
                next_tx,
                closure.id.clone(),
                Ident::new(&format!("pods_{}", closure.id), Span::call_site()),
                output_type,
                routing_targets,
                step_constraints,
            ))
        })
        .collect::<Vec<Result<TokenStream>>>()
        .into_iter()
        .collect::<Result<Vec<TokenStream>>>()?;

    let pool_request_ty = match closure_steps.first().ok_or(anyhow!("No closure"))? {
        closure => {
            if closure.arg_ty.len() == 1 {
                let arg_ty = closure
                    .arg_ty
                    .first()
                    .cloned()
                    .ok_or(anyhow!("First node is closure but arg_ty is empty"))?;
                quote! { #arg_ty }
            } else {
                let arg_ty = closure.arg_ty.clone();
                quote! { (#( #arg_ty ),*) }
            }
        }
    };
    let pool_response_ty = match closure_steps.last().ok_or(anyhow!("No closure"))? {
        closure => {
            if closure.ret_ty.path.segments.len() == 1 {
                let ret_ty = closure
                    .ret_ty
                    .clone()
                    .path
                    .segments
                    .first()
                    .cloned()
                    .ok_or(anyhow!("Last node is closure but ret_ty is empty"))?;
                quote! { #ret_ty }
            } else {
                let ret_ty = closure.ret_ty.clone();
                quote! { #ret_ty }
            }
        }
    };

    // Generate max_thread_count expression based on global constraints
    let global_max_threads_expr = if let Some(global_constraints) = &global_constraints {
        if let Some(max_threads) = &global_constraints.max_threads {
            quote! { #max_threads }
        } else {
            quote! { num_cpus::get() }
        }
    } else {
        quote! { num_cpus::get() }
    };

    Ok(quote! {
        use ::ichika::status::IntoStatus;

        struct _Pool {
            daemon: Option<::std::thread::JoinHandle<::ichika::anyhow::Result<()>>>,
            tx_shutdown: ::ichika::flume::Sender<()>,

            tx_send_request: ::ichika::flume::Sender<#pool_request_ty>,
            rx_recv_response: ::ichika::flume::Receiver<#pool_response_ty>,

            tx_thread_usage_request: ::ichika::flume::Sender<()>,
            rx_thread_usage_response: ::ichika::flume::Receiver<usize>,
            tx_task_count_request: ::ichika::flume::Sender<String>,
            rx_task_count_response: ::ichika::flume::Receiver<usize>,
        }

        impl ::ichika::pool::ThreadPool for _Pool {
            type Request = #pool_request_ty;
            type Response = #pool_response_ty;

            fn send(&self, req: Self::Request) -> ::ichika::anyhow::Result<()> {
                self.tx_send_request.send(req)?;
                Ok(())
            }

            fn recv(&self) -> ::ichika::anyhow::Result<Option<Self::Response>> {
                Ok(self
                    .rx_recv_response
                    .try_recv()
                    .map(|res| Some(res.to_owned()))
                    .unwrap_or_default())
            }

            fn thread_usage(&self) -> ::ichika::anyhow::Result<usize> {
                self.tx_thread_usage_request.send(())?;
                self.rx_thread_usage_response
                    .recv()
                    .map_err(|_| ::ichika::anyhow::anyhow!("No response"))
            }
            fn task_count(&self, stage: impl ToString) -> ::ichika::anyhow::Result<usize> {
                self.tx_task_count_request.send(stage.to_string())?;
                self.rx_task_count_response
                    .recv()
                    .map_err(|_| ::ichika::anyhow::anyhow!("No response"))
            }
        }

        impl _Pool {
            pub fn new() -> ::ichika::anyhow::Result<Self> {
                use ::ichika::{node::*, pod::ThreadPod};

                let (tx_shutdown, rx_shutdown) = ::ichika::flume::bounded(1);
                let (tx_thread_usage_request, rx_thread_usage_request) = ::ichika::flume::bounded(1);
                let (tx_thread_usage_response, rx_thread_usage_response) = ::ichika::flume::bounded(1);
                let (tx_task_count_request, rx_task_count_request) = ::ichika::flume::bounded(1);
                let (tx_task_count_response, rx_task_count_response) = ::ichika::flume::bounded(1);

                #( #tx_rx_request_flume_unbounded )*
                let (tx_pods_response, rx_pods_response) = ::ichika::flume::unbounded::<#pool_response_ty>();

                let daemon = std::thread::spawn({
                    let #flume_request_unbounded_first_ident = #flume_request_unbounded_first_ident.clone();

                    move || {
                        // Apply global max_threads constraint if provided
                        let max_thread_count = #global_max_threads_expr;
                        #pods_init_let

                        loop {
                            #( #clean_pods_code )*

                            let prev_pods_size = 0;
                            #( #thread_creators )*

                            if rx_thread_usage_request.try_recv().is_ok() {
                                tx_thread_usage_response
                                    .send(#( #calculate_pods_len_code )+*)
                                    .unwrap();
                            }
                            if rx_task_count_request.try_recv().is_ok() {
                                tx_task_count_response
                                    .send(
                                        #( #calculate_pods_len_code )+*
                                            + #flume_request_unbounded_first_ident.len(),
                                    )
                                    .unwrap();
                            }
                            if rx_shutdown.try_recv().is_ok() {
                                break;
                            }

                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }

                        loop {
                            #( #clean_pods_code )*

                            if #(#has_all_pods_stop_code)&&* {
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                        ::ichika::anyhow::Ok(())
                    }
                });

                Ok(Self {
                    daemon: Some(daemon),
                    tx_shutdown,

                    tx_send_request: #flume_request_unbounded_first_ident.clone(),
                    rx_recv_response: rx_pods_response,

                    tx_thread_usage_request,
                    rx_thread_usage_response,
                    tx_task_count_request,
                    rx_task_count_response,
                })
            }
        }

        impl Drop for _Pool {
            fn drop(&mut self) {
                self.tx_shutdown.send(()).unwrap();
                self.daemon.take().unwrap().join().unwrap().unwrap();
            }
        }
    })
}
