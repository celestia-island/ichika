use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn generate_thread_creator(
    rx_request: Ident,
    tx_response: Ident,
    target_step_ident: Ident,
    target_step_pods_ident: Ident,
) -> Result<TokenStream> {
    // 这里的当前线程数量，默认情况下为根据该阶段任务及其后续所有任务的总数来决定
    // 例如，如果有三个阶段的当前任务数量 a b c，
    // 那么第一个任务的数量为 a + b + c，第二个任务的数量为 b + c，第三个任务的数量为 c
    // 这么做是为了保证每个阶段的任务数量是平均的，不会因为前一阶段的任务数量过多而导致后一阶段的任务数量过少
    // 后续开放的自定义数量的接口，设定的数值除了总线程数量和每个阶段的固定线程数外
    // 还可以设定为每个阶段不低于/不高于若干个线程
    // （会配合这里的 len() 累加，例如限制每一阶段至少预留两个，那么第一阶段就是 a + b + c + 2 * 2）

    Ok(quote! {
        if !#rx_request.is_empty()
            && prev_pods_size + #target_step_pods_ident.len() < max_thread_count
        {
            let thread = std::thread::spawn({
                let rx_request = #rx_request.clone();
                let tx_response = #tx_response.clone();

                move || {
                    while let Ok(req) = rx_request.try_recv() {
                        let res = #target_step_ident::run(req);
                        match res {
                            ::ichika::Status::Next(res) => tx_response.send(res).unwrap(),
                            _ => {
                                todo!();
                            }
                        };
                    }
                    ::ichika::anyhow::Ok(())
                }
            });
            #target_step_pods_ident.push(ThreadPod::new(#target_step_ident::id(), thread));
        }

        let prev_pods_size = prev_pods_size + #target_step_pods_ident.len();
    })
}
