use anyhow::{anyhow, Result};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::tools::{
    pipe::PipeNode,
    pipe_flatten::{ClosureMacrosFlatten, PipeNodeFlatten},
};

fn generate_thread_creator(index: usize, closure: ClosureMacrosFlatten) -> Result<TokenStream> {
    // 这里的当前线程数量，默认情况下为根据该阶段任务及其后续所有任务的总数来决定
    // 例如，如果有三个阶段的当前任务数量 a b c，
    // 那么第一个任务的数量为 a + b + c，第二个任务的数量为 b + c，第三个任务的数量为 c
    // 这么做是为了保证每个阶段的任务数量是平均的，不会因为前一阶段的任务数量过多而导致后一阶段的任务数量过少
    // 后续开放的自定义数量的接口，设定的数值除了总线程数量和每个阶段的固定线程数外
    // 还可以设定为每个阶段不低于/不高于若干个线程
    // （会配合这里的 len() 累加，例如限制每一阶段至少预留两个，那么第一阶段就是 a + b + c + 2 * 2）
    let next_index = index + 1;
    let ClosureMacrosFlatten { id, .. } = closure;

    Ok(quote! {
        if !flume_request_unbounded[#index].1.is_empty()
            && pods.iter().skip(#index).map(|pods| pods.len()).reduce(|prev, next| prev + next).unwrap_or(0) < max_thread_count
        {
            let thread = std::thread::spawn({
                let rx_request = flume_request_unbounded[#index].1.clone();
                let tx_response = flume_request_unbounded[#next_index].0.clone();

                move || {
                    while let Ok(req) = rx_request.try_recv() {
                        let res = #id::run(req);
                        match res {
                            Ok(res) => tx_response.send(res).unwrap(),
                            Err(err) => {
                                todo!("Switch to error channel: {:?}", err);
                            }
                        };
                    }
                    ::ichika::Ok(())
                }
            });
            pods[#index].push(ThreadPod::new(#id::id(), thread));
        }
    })
}

pub(crate) fn generate_pool(closures: Vec<PipeNodeFlatten>) -> Result<TokenStream> {
    let tx_rx_request_flume_unbounded = closures
        .iter()
        .map(|_| {
            quote! {
                ::ichika::flume::unbounded()
            }
        })
        .collect::<Vec<TokenStream>>();
    let tx_rx_request_flume_unbounded = quote! {
        let flume_request_unbounded = [ #(#tx_rx_request_flume_unbounded),* ];
    };

    let pods_init_let = closures
        .iter()
        .map(|_| {
            quote! { vec![] }
        })
        .collect::<Vec<TokenStream>>();
    let pods_init_let = quote! {
        let mut pods = [ #(#pods_init_let),* ];
    };

    let thread_creators = closures
        .iter()
        .enumerate()
        .map(|(index, step)| match step {
            PipeNodeFlatten::Closure(closure) => generate_thread_creator(index, closure.clone()),
            PipeNodeFlatten::Map(_) => todo!(),
        })
        .collect::<Vec<Result<TokenStream>>>();
    let thread_creators = thread_creators
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
        struct _Pool {
            daemon: Option<::std::thread::JoinHandle<::ichika::Result<()>>>,
            tx_shutdown: ::ichika::flume::Sender<()>,

            tx_send_request: ::ichika::flume::Sender<<Self as ::ichika::pool::ThreadPool>::Request>,
            rx_recv_response: ::ichika::flume::Receiver<<Self as ::ichika::pool::ThreadPool>::Response>,

            tx_thread_usage_request: ::ichika::flume::Sender<()>,
            rx_thread_usage_response: ::ichika::flume::Receiver<usize>,
            tx_task_count_request: ::ichika::flume::Sender<()>,
            rx_task_count_response: ::ichika::flume::Receiver<usize>,
        }

        impl ::ichika::pool::ThreadPool for _Pool {
            type Request = #pool_request_ty;
            type Response = #pool_response_ty;

            fn send(&self, req: Self::Request) -> ::ichika::Result<()> {
                self.tx_send_request.send(req)?;
                Ok(())
            }

            fn recv(&self) -> ::ichika::Result<Option<Self::Response>> {
                Ok(self
                    .rx_recv_response
                    .try_recv()
                    .map(|res| Some(res.to_owned()))
                    .unwrap_or_default())
            }

            fn thread_usage(&self) -> ::ichika::Result<usize> {
                self.tx_thread_usage_request.send(())?;
                self.rx_thread_usage_response
                    .recv()
                    .map_err(|_| ::ichika::anyhow!("No response"))
            }
            fn task_count(&self, stage: impl ToString) -> ::ichika::Result<usize> {
                self.tx_task_count_request.send(stage.to_string())?;
                self.rx_task_count_response
                    .recv()
                    .map_err(|_| ::ichika::anyhow!("No response"))
            }
        }

        impl _Pool {
            pub fn new() -> ::ichika::Result<Self> {
                use ::ichika::{node::*, pod::ThreadPod};

                let (tx_shutdown, rx_shutdown) = ::ichika::flume::bounded(1);
                let (tx_thread_usage_request, rx_thread_usage_request) = ::ichika::flume::bounded(1);
                let (tx_thread_usage_response, rx_thread_usage_response) = ::ichika::flume::bounded(1);
                let (tx_task_count_request, rx_task_count_request) = ::ichika::flume::bounded(1);
                let (tx_task_count_response, rx_task_count_response) = ::ichika::flume::bounded(1);

                #tx_rx_request_flume_unbounded
                let (tx_pods_response, rx_pods_response) = ::ichika::flume::unbounded();

                let daemon = std::thread::spawn(move || {
                    // TODO: Read from outside
                    let max_thread_count = num_cpus::get();
                    #pods_init_let

                    loop {
                        // Clean all finished threads
                        pods.iter_mut().map(|pods| {
                            pods.retain(|pod: &ThreadPod| pod.is_alive());
                        });

                        #(#thread_creators)*

                        if rx_thread_usage_request.try_recv().is_ok() {
                            tx_thread_usage_response
                                .send(pods.iter().map(|pods| pods.len()).reduce(|prev, next| prev + next)).unwrap_or(0)
                                .unwrap();
                        }
                        if rx_task_count_request.try_recv().is_ok() {
                            tx_task_count_response
                                .send(
                                    pods.iter().map(|pods| pods.len()).reduce(|prev, next| prev + next).unwrap_or(0)
                                        + flume_request_unbounded[0].1.len(),
                                )
                                .unwrap();
                        }
                        if rx_shutdown.try_recv().is_ok() {
                            break;
                        }

                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }

                    loop {
                        pods.iter_mut().map(|pods| {
                            pods.retain(|pod: &ThreadPod| pod.is_alive());
                        });

                        if pods.iter().all(|pods| pods.is_empty()) {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    anyhow::Ok(())
                });

                Ok(Self {
                    daemon: Some(daemon),
                    tx_shutdown,

                    tx_send_request: flume_request_unbounded[0].0.clone(),
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
