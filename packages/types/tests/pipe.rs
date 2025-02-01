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

            tx_thread_usage_request: flume::Sender<()>,
            rx_thread_usage_response: flume::Receiver<usize>,
            tx_task_count_request: flume::Sender<String>,
            rx_task_count_response: flume::Receiver<usize>,
        }
        impl ::ichika::pool::ThreadPool for _Pool {
            type Request = String;
            type Response = String;

            fn send(&self, req: Self::Request) -> Result<()> {
                self.tx_send_request.send(::std::sync::Arc::new(req))?;
                Ok(())
            }

            fn recv(&self) -> Result<Option<Self::Response>> {
                self.rx_recv_response
                    .try_recv()
                    .map(|res| Some(res.as_ref().to_owned()))
                    .map_err(|_| anyhow!("No response"))
            }

            fn thread_usage(&self) -> Result<usize> {
                self.tx_thread_usage_request.send(())?;
                self.rx_thread_usage_response
                    .recv()
                    .map_err(|_| anyhow!("No response"))
            }
            fn task_count(&self, stage: impl ToString) -> Result<usize> {
                self.tx_task_count_request.send(stage.to_string())?;
                self.rx_task_count_response
                    .recv()
                    .map_err(|_| anyhow!("No response"))
            }
        }

        impl _Pool {
            pub fn new() -> Result<Self> {
                let (tx_send_request, rx_send_request) = flume::unbounded();
                let (tx_recv_response, rx_recv_response) = flume::unbounded();

                let (tx_shutdown, rx_shutdown) = flume::bounded(1);
                let (tx_thread_usage_request, rx_thread_usage_request) = flume::bounded(1);
                let (tx_thread_usage_response, rx_thread_usage_response) = flume::bounded(1);
                let (tx_task_count_request, rx_task_count_request) = flume::bounded(1);
                let (tx_task_count_response, rx_task_count_response) = flume::bounded(1);

                let daemon = std::thread::spawn(move || {
                    let mut pods = vec![];

                    loop {
                        if rx_shutdown.try_recv().is_ok() {
                            break;
                        }

                        if rx_send_request.is_empty() {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            continue;
                        }

                        if rx_thread_usage_request.try_recv().is_ok() {
                            // TODO: Not done yet
                            tx_thread_usage_response.send(pods.len()).unwrap();
                        }
                        if rx_task_count_request.try_recv().is_ok() {
                            // TODO: Not done yet
                            tx_task_count_response.send(pods.len()).unwrap();
                        }
                    }

                    anyhow::Ok(())
                });

                Ok(Self {
                    daemon: Some(daemon),
                    tx_shutdown,

                    tx_send_request,
                    rx_recv_response,

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
    }?;

    Ok(())
}
