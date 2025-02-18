use anyhow::{anyhow, Result};

use ichika::{pipe, pod::ThreadPod, pool::ThreadPool};

#[test]
fn create_pipe() -> Result<()> {
    // let pool = pipe![
    //     |req: String| -> Result<usize> { Ok(req.len()) },
    //     |req: usize| -> Result<String> { Ok(req.to_string()) }
    // ];
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    log::info!("Create pool");
    let pool = {
        // generate_closures
        struct _Stage0_0;
        struct _Stage1_0;

        impl ::ichika::node::ThreadNode for _Stage0_0 {
            type Request = String;
            type Response = usize;

            fn run(req: Self::Request) -> Self::Response {
                std::thread::sleep(std::time::Duration::from_millis(rand::random_range(
                    500..3000,
                )));
                req.len()
            }
        }
        impl ::ichika::node::ThreadNode for _Stage1_0 {
            type Request = usize;
            type Response = String;

            fn run(req: Self::Request) -> Self::Response {
                std::thread::sleep(std::time::Duration::from_millis(rand::random_range(
                    500..3000,
                )));
                req.to_string()
            }
        }

        impl ::ichika::node::ThreadNodeEnum for _Stage0_0 {
            fn id() -> &'static str {
                "Stage0_0"
            }
        }
        impl ::ichika::node::ThreadNodeEnum for _Stage1_0 {
            fn id() -> &'static str {
                "Stage1_0"
            }
        }

        // generate_pool
        struct _Pool {
            daemon: Option<::std::thread::JoinHandle<::anyhow::Result<()>>>,
            tx_shutdown: flume::Sender<()>,

            tx_send_request: flume::Sender<<Self as ::ichika::pool::ThreadPool>::Request>,
            rx_recv_response: flume::Receiver<<Self as ::ichika::pool::ThreadPool>::Response>,

            tx_thread_usage_request: flume::Sender<()>,
            rx_thread_usage_response: flume::Receiver<usize>,
            tx_task_count_request: flume::Sender<String>,
            rx_task_count_response: flume::Receiver<usize>,
        }
        impl ::ichika::pool::ThreadPool for _Pool {
            type Request = String;
            type Response = String;

            fn send(&self, req: Self::Request) -> Result<()> {
                self.tx_send_request.send(req)?;
                Ok(())
            }

            fn recv(&self) -> Result<Option<Self::Response>> {
                Ok(self
                    .rx_recv_response
                    .try_recv()
                    .map(|res| Some(res.to_owned()))
                    .unwrap_or_default())
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
                use ::ichika::{node::*, pod::ThreadPod};

                let (tx_shutdown, rx_shutdown) = flume::bounded(1);
                let (tx_thread_usage_request, rx_thread_usage_request) = flume::bounded(1);
                let (tx_thread_usage_response, rx_thread_usage_response) = flume::bounded(1);
                let (tx_task_count_request, rx_task_count_request) = flume::bounded(1);
                let (tx_task_count_response, rx_task_count_response) = flume::bounded(1);

                let (tx_pods_stage0_0_request, rx_pods_stage0_0_request) = flume::unbounded();
                let (tx_pods_stage1_0_request, rx_pods_stage1_0_request) = flume::unbounded();
                let (tx_pods_response, rx_pods_response) = flume::unbounded();

                let daemon = std::thread::spawn(move || {
                    log::info!("Daemon thread is starting");

                    // TODO: Read from outside
                    let max_thread_count = num_cpus::get();
                    log::info!("max_thread_count: {}", max_thread_count);
                    let mut pods_stage0_0 = vec![];
                    let mut pods_stage1_0 = vec![];

                    loop {
                        // Clean all finished threads
                        log::info!(
                            "Before clean: Stage0_0: {}, Stage1_0: {}",
                            pods_stage0_0.len(),
                            pods_stage1_0.len()
                        );
                        pods_stage0_0.retain(|pod: &ThreadPod| pod.is_alive());
                        pods_stage1_0.retain(|pod: &ThreadPod| pod.is_alive());
                        log::info!(
                            "After clean: Stage0_0: {}, Stage1_0: {}",
                            pods_stage0_0.len(),
                            pods_stage1_0.len()
                        );

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
                            log::info!("Spawn thread for Stage0_0");
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
                            log::info!("Spawn thread for Stage1_0");
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
                                .send(pods_stage0_0.len() + pods_stage1_0.len())
                                .unwrap();
                        }
                        if rx_task_count_request.try_recv().is_ok() {
                            tx_task_count_response
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

                        log::info!("Daemon thread is running");
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }

                    log::info!("Clean up all threads");
                    loop {
                        pods_stage0_0.retain(|pod| pod.is_alive());
                        pods_stage1_0.retain(|pod| pod.is_alive());

                        log::info!(
                            "After clean: Stage0_0: {}, Stage1_0: {}",
                            pods_stage0_0.len(),
                            pods_stage1_0.len()
                        );
                        if pods_stage0_0.is_empty() && pods_stage1_0.is_empty() {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    log::info!("Daemon thread is exiting");
                    anyhow::Ok(())
                });

                Ok(Self {
                    daemon: Some(daemon),
                    tx_shutdown,

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
                log::info!("Pool is dropped");
            }
        }

        _Pool::new()
    }?;
    log::info!("Pool is created");

    // Test case
    // Generate some random string with random length
    const TEST_CASE_MAX_COUNT: usize = 10;
    for i in 0..TEST_CASE_MAX_COUNT {
        for j in 0..TEST_CASE_MAX_COUNT {
            let req = (i..TEST_CASE_MAX_COUNT)
                .map(|_| ('a' as u8 + j as u8) as char)
                .collect::<String>();
            log::info!("Send: {:?}", req);
            pool.send(req)?;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            log::info!("Receive: {:?}", res);
        } else {
            break;
        }
    }
    log::info!("All responses are received");

    Ok(())
}
