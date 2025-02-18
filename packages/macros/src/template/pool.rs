use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use crate::tools::{pipe::PipeNode, PipeMacros};

fn generate_pods_request_flume_unbounded(stages: Vec<PipeNode>) -> Result<TokenStream> {
    let stage_request_flume_unbounded = stages
        .iter()
        .map(|closure| todo!("递归调用自己"))
        .collect::<Vec<_>>();

    Ok(quote! {
        // #(#stage_request_flume_unbounded)*
    })
}

pub(crate) fn generate_pool(funcs: PipeMacros) -> Result<TokenStream> {
    Ok(quote! {
        impl _Pool {
            pub fn new() -> ::ichika::Result<Self> {
                use ::ichika::{node::*, pod::ThreadPod};

                let (tx_shutdown, rx_shutdown) = ::ichika::flume::bounded(1);
                let (tx_thread_usage_request, rx_thread_usage_request) = ::ichika::flume::bounded(1);
                let (tx_thread_usage_response, rx_thread_usage_response) = ::ichika::flume::bounded(1);
                let (tx_task_count_request, rx_task_count_request) = ::ichika::flume::bounded(1);
                let (tx_task_count_response, rx_task_count_response) = ::ichika::flume::bounded(1);

                // TODO: Dynamic generate
                let (tx_pods_stage0_0_request, rx_pods_stage0_0_request) = flume::unbounded();
                let (tx_pods_stage1_0_request, rx_pods_stage1_0_request) = flume::unbounded();
                let (tx_pods_response, rx_pods_response) = ::ichika::flume::unbounded();

                let daemon = std::thread::spawn(move || {
                    // TODO: Read from outside
                    let max_thread_count = num_cpus::get();
                    // TODO: Dynamic generate
                    let mut pods_stage0_0 = vec![];
                    let mut pods_stage1_0 = vec![];

                    loop {
                        // Clean all finished threads
                        // TODO: Dynamic generate
                        pods_stage0_0.retain(|pod: &ThreadPod| pod.is_alive());
                        pods_stage1_0.retain(|pod: &ThreadPod| pod.is_alive());

                        // TODO: Dynamic generate
                        if !rx_pods_stage0_0_request.is_empty()
                            && pods_stage0_0.len() + pods_stage1_0.len() < max_thread_count
                        // 这里的当前线程数量，默认情况下为根据该阶段任务及其后续所有任务的总数来决定
                        // 例如，如果有三个阶段的当前任务数量 a b c，
                        // 那么第一个任务的数量为 a + b + c，第二个任务的数量为 b + c，第三个任务的数量为 c
                        // 这么做是为了保证每个阶段的任务数量是平均的，不会因为前一阶段的任务数量过多而导致后一阶段的任务数量过少
                        // 后续开放的自定义数量的接口，设定的数值除了总线程数量和每个阶段的固定线程数外
                        // 还可以设定为每个阶段不低于/不高于若干个线程
                        // （会配合这里的 len() 累加，例如限制每一阶段至少预留两个，那么第一阶段就是 a + b + c + 2 * 2）
                        {
                            let thread = std::thread::spawn({
                                let rx_request = rx_pods_stage0_0_request.clone();
                                let tx_response = tx_pods_stage1_0_request.clone();

                                move || {
                                    while let Ok(req) = rx_request.try_recv() {
                                        let res = _Stage0_0::run(req);
                                        tx_response.send(res).unwrap();
                                    }
                                    anyhow::Ok(())
                                }
                            });
                            pods_stage0_0.push(ThreadPod::new(_Stage0_0::id(), thread));
                        }
                        if !rx_pods_stage1_0_request.is_empty()
                            && pods_stage1_0.len() < max_thread_count
                        {
                            let thread = std::thread::spawn({
                                let rx_request = rx_pods_stage1_0_request.clone();
                                let tx_response = tx_pods_response.clone();

                                move || {
                                    while let Ok(req) = rx_request.try_recv() {
                                        let res = _Stage1_0::run(req);
                                        tx_response.send(res).unwrap();
                                    }
                                    anyhow::Ok(())
                                }
                            });
                            pods_stage1_0.push(ThreadPod::new(_Stage1_0::id(), thread));
                        }

                        if rx_thread_usage_request.try_recv().is_ok() {
                            tx_thread_usage_response
                                // TODO: Dynamic generate
                                .send(pods_stage0_0.len() + pods_stage1_0.len())
                                .unwrap();
                        }
                        if rx_task_count_request.try_recv().is_ok() {
                            tx_task_count_response
                                // TODO: Dynamic generate
                                .send(
                                    pods_stage0_0.len()
                                        + pods_stage1_0.len()
                                        + rx_pods_stage0_0_request.len(),
                                )
                                .unwrap();
                        }
                        if rx_shutdown.try_recv().is_ok() {
                            break;
                        }

                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }

                    loop {
                        // TODO: Dynamic generate
                        pods_stage0_0.retain(|pod| pod.is_alive());
                        pods_stage1_0.retain(|pod| pod.is_alive());

                        if pods_stage0_0.is_empty() && pods_stage1_0.is_empty() {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    anyhow::Ok(())
                });

                Ok(Self {
                    daemon: Some(daemon),
                    tx_shutdown,

                    // TODO: Dynamic generate
                    tx_send_request: tx_pods_stage0_0_request,
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

        _Pool::new()
    })
}
