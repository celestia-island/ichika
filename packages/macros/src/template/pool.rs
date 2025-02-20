use anyhow::{anyhow, Result};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::tools::pipe_flatten::PipeNodeFlatten;

use super::generate_thread_creator;

pub(crate) fn generate_pool(closures: Vec<PipeNodeFlatten>) -> Result<TokenStream> {
    let tx_rx_request_flume_unbounded = closures
        .iter()
        .map(|step| {
            let name = match step {
                PipeNodeFlatten::Closure(closure) => closure.id.clone(),
                PipeNodeFlatten::Map(_) => todo!(),
            };

            let tx_ident = Ident::new(&format!("tx_{}", name), Span::call_site());
            let rx_ident = Ident::new(&format!("rx_{}", name), Span::call_site());
            quote! {
               let (#tx_ident, #rx_ident) = ::ichika::flume::unbounded();
            }
        })
        .collect::<Vec<TokenStream>>();
    let flume_request_unbounded_first_ident = match closures.first().ok_or(anyhow!("No closure"))? {
        PipeNodeFlatten::Closure(closure) => {
            Ident::new(&format!("tx_{}", closure.id), Span::call_site())
        }
        PipeNodeFlatten::Map(_) => return Err(anyhow!("First node is not closure")),
    };

    let pods_name = closures
        .iter()
        .map(|pods| match pods {
            PipeNodeFlatten::Closure(closure) => {
                Ident::new(&format!("pods_{}", closure.id), Span::call_site())
            }
            PipeNodeFlatten::Map(_) => todo!(),
        })
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

    let thread_creators = closures
        .iter()
        .zip(
            closures
                .iter()
                .skip(1)
                .map(|next_step| {
                    Ident::new(
                        &format!(
                            "tx_{}",
                            match next_step {
                                PipeNodeFlatten::Closure(closure) => closure.id.clone(),
                                PipeNodeFlatten::Map(_) => todo!(),
                            }
                        ),
                        Span::call_site(),
                    )
                })
                .chain([Ident::new("tx_pods_response", Span::call_site())])
                .collect::<Vec<_>>()
                .iter(),
        )
        .rev()
        .map(|(step, tx_response)| match step {
            PipeNodeFlatten::Closure(closure) => generate_thread_creator(
                Ident::new(&format!("rx_{}", closure.id), Span::call_site()),
                tx_response.to_owned(),
                closure.id.clone(),
                Ident::new(&format!("pods_{}", closure.id), Span::call_site()),
            ),
            PipeNodeFlatten::Map(_) => todo!(),
        })
        .collect::<Vec<Result<TokenStream>>>()
        .into_iter()
        .collect::<Result<Vec<TokenStream>>>()?;

    let pool_request_ty = match closures.first().ok_or(anyhow!("No closure"))? {
        PipeNodeFlatten::Closure(closure) => closure.arg_ty.clone(),
        PipeNodeFlatten::Map(_) => return Err(anyhow!("First node is not closure")),
    };
    let pool_response_ty = match closures.last().ok_or(anyhow!("No closure"))? {
        PipeNodeFlatten::Closure(closure) => closure.ret_ty.clone(),
        PipeNodeFlatten::Map(_) => return Err(anyhow!("Last node is not closure")),
    };

    Ok(quote! {
        use ::ichika::status::IntoStatus;

        struct _Pool {
            daemon: Option<::std::thread::JoinHandle<::ichika::anyhow::Result<()>>>,
            tx_shutdown: ::ichika::flume::Sender<()>,

            tx_send_request: ::ichika::flume::Sender<<Self as ::ichika::pool::ThreadPool>::Request>,
            rx_recv_response: ::ichika::flume::Receiver<<Self as ::ichika::pool::ThreadPool>::Response>,

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
                let (tx_pods_response, rx_pods_response) = ::ichika::flume::unbounded();

                let daemon = std::thread::spawn({
                    let #flume_request_unbounded_first_ident = #flume_request_unbounded_first_ident.clone();

                    move || {
                        // TODO: Read from outside
                        let max_thread_count = num_cpus::get();
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
