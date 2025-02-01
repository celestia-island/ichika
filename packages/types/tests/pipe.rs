use anyhow::{anyhow, Result};

use ichika::pipe;

#[test]
fn create_pipe() -> Result<()> {
    // let pool = pipe![
    //     |req: String| -> Result<usize> { Ok(req.len()) },
    //     |req: usize| -> Result<String> { Ok(req.to_string()) }
    // ];

    let pool = {
        struct _Stage0_0;
        struct _Stage1_0;

        impl ::ichika::node::ThreadNode for _Stage0_0 {
            type Request = String;
            type Response = usize;

            fn run(req: Self::Request) -> Self::Response {
                req.len()
            }
        }
        impl ::ichika::node::ThreadNode for _Stage1_0 {
            type Request = usize;
            type Response = String;

            fn run(req: Self::Request) -> Self::Response {
                req.to_string()
            }
        }

        impl ::ichika::node::ThreadNodeEnum for _Stage0_0 {
            fn id(&self) -> &str {
                "Stage0_0"
            }
        }
        impl ::ichika::node::ThreadNodeEnum for _Stage1_0 {
            fn id(&self) -> &str {
                "Stage1_0"
            }
        }

        struct _Pool {
            daemon: Option<::std::thread::JoinHandle<::anyhow::Result<()>>>,
            tx_shutdown: flume::Sender<()>,

            tx_send_request:
                flume::Sender<::std::sync::Arc<<Self as ::ichika::pool::ThreadPool>::Request>>,
            rx_recv_response:
                flume::Receiver<::std::sync::Arc<<Self as ::ichika::pool::ThreadPool>::Response>>,
        }
        impl ::ichika::pool::ThreadPool for _Pool {
            type Request = String;
            type Response = String;

            fn send(&mut self, req: Self::Request) {
                // TODO: Not done yet
                todo!()
            }

            fn recv(&mut self) -> Option<Self::Response> {
                // TODO: Not done yet
                todo!()
            }

            fn thread_usage(&self) -> usize {
                // TODO: 通过管道通信获取目前线程池持有的总线程数量，记得加上守护线程
                0
            }
            fn task_count(&self, stage: impl ToString) -> usize {
                // TODO: 读取每个 pod 下持有 flume 句柄的积压数据条目数量
                // TODO: 通过管道通信获取每个 pod 下的任务数量
                0
            }
        }

        impl _Pool {
            pub fn new() -> Result<Self> {
                let (tx_send_request, rx_send_request) = flume::unbounded();
                let (tx_recv_response, rx_recv_response) = flume::unbounded();
                let (tx_shutdown, rx_shutdown) = flume::bounded(1);

                let daemon = std::thread::spawn(move || {
                    let mut pods = vec![];

                    loop {
                        if rx_shutdown.try_recv().is_ok() {
                            break;
                        }
                    }

                    anyhow::Ok(())
                });

                Ok(Self {
                    daemon: Some(daemon),
                    tx_shutdown,

                    tx_send_request,
                    rx_recv_response,
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
    }?;

    Ok(())
}
