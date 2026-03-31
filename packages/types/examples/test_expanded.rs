#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2021::*;
use ichika::pipe;
fn main() {
    let _ = {
        struct _step_0;
        impl ::ichika::node::ThreadNode for _step_0 {
            type Request = String;
            type Response = usize;
            fn run(
                req: Self::Request,
            ) -> ::ichika::Status<Self::Response, ::ichika::anyhow::Error> {
                { Ok(req.len()) }.into_status()
            }
        }
        impl ::ichika::node::ThreadNodeEnum for _step_0 {
            fn id() -> &'static str {
                "_step_0"
            }
        }
        struct _step_1;
        impl ::ichika::node::ThreadNode for _step_1 {
            type Request = usize;
            type Response = String;
            fn run(
                req: Self::Request,
            ) -> ::ichika::Status<Self::Response, ::ichika::anyhow::Error> {
                { Ok(req.to_string()) }.into_status()
            }
        }
        impl ::ichika::node::ThreadNodeEnum for _step_1 {
            fn id() -> &'static str {
                "_step_1"
            }
        }
        use ::ichika::status::IntoStatus;
        struct _Pool {
            daemon: Option<::std::thread::JoinHandle<::ichika::anyhow::Result<()>>>,
            tx_shutdown: ::ichika::flume::Sender<()>,
            tx_send_request: ::ichika::flume::Sender<String>,
            rx_recv_response: ::ichika::flume::Receiver<String>,
            tx_thread_usage_request: ::ichika::flume::Sender<()>,
            rx_thread_usage_response: ::ichika::flume::Receiver<usize>,
            tx_task_count_request: ::ichika::flume::Sender<String>,
            rx_task_count_response: ::ichika::flume::Receiver<usize>,
        }
        impl ::ichika::pool::ThreadPool for _Pool {
            type Request = String;
            type Response = String;
            fn send(&self, req: Self::Request) -> ::ichika::anyhow::Result<()> {
                self.tx_send_request.send(req)?;
                Ok(())
            }
            fn recv(&self) -> ::ichika::anyhow::Result<Option<Self::Response>> {
                Ok(
                    self
                        .rx_recv_response
                        .try_recv()
                        .map(|res| Some(res.to_owned()))
                        .unwrap_or_default(),
                )
            }
            fn thread_usage(&self) -> ::ichika::anyhow::Result<usize> {
                self.tx_thread_usage_request.send(())?;
                self.rx_thread_usage_response
                    .recv()
                    .map_err(|_| ::anyhow::__private::must_use({
                        let error = ::anyhow::__private::format_err(
                            format_args!("No response"),
                        );
                        error
                    }))
            }
            fn task_count(
                &self,
                stage: impl ToString,
            ) -> ::ichika::anyhow::Result<usize> {
                self.tx_task_count_request.send(stage.to_string())?;
                self.rx_task_count_response
                    .recv()
                    .map_err(|_| ::anyhow::__private::must_use({
                        let error = ::anyhow::__private::format_err(
                            format_args!("No response"),
                        );
                        error
                    }))
            }
        }
        impl _Pool {
            pub fn new() -> ::ichika::anyhow::Result<Self> {
                use ::ichika::{node::*, pod::ThreadPod};
                let (tx_shutdown, rx_shutdown) = ::ichika::flume::bounded(1);
                let (tx_thread_usage_request, rx_thread_usage_request) = ::ichika::flume::bounded(
                    1,
                );
                let (tx_thread_usage_response, rx_thread_usage_response) = ::ichika::flume::bounded(
                    1,
                );
                let (tx_task_count_request, rx_task_count_request) = ::ichika::flume::bounded(
                    1,
                );
                let (tx_task_count_response, rx_task_count_response) = ::ichika::flume::bounded(
                    1,
                );
                let (tx__step_0, rx__step_0) = ::ichika::flume::unbounded::<String>();
                let (tx__step_1, rx__step_1) = ::ichika::flume::unbounded::<usize>();
                let (tx_pods_response, rx_pods_response) = ::ichika::flume::unbounded::<
                    String,
                >();
                let daemon = std::thread::spawn({
                    let tx__step_0 = tx__step_0.clone();
                    move || {
                        let max_thread_count = num_cpus::get();
                        let mut pods__step_0 = Vec::new();
                        let mut pods__step_1 = Vec::new();
                        loop {
                            pods__step_0.retain(|pod: &ThreadPod| pod.is_alive());
                            pods__step_1.retain(|pod: &ThreadPod| pod.is_alive());
                            let prev_pods_size = 0;
                            if !rx__step_0.is_empty()
                                && prev_pods_size + pods__step_0.len() < max_thread_count
                            {
                                let thread = std::thread::spawn({
                                    let rx_request = rx__step_0.clone();
                                    let tx_response = tx__step_1.clone();
                                    let mut routing_table: ::std::collections::HashMap<
                                        &'static str,
                                        ::ichika::flume::Sender<usize>,
                                    > = ::std::collections::HashMap::new();
                                    routing_table.insert("_step_1", tx__step_1.clone());
                                    move || {
                                        while let Ok(mut req) = rx_request.try_recv() {
                                            let mut attempt: usize = 0;
                                            loop {
                                                let res = _step_0::run(req);
                                                match res {
                                                    ::ichika::Status::Next(res) => {
                                                        tx_response.send(res).unwrap();
                                                        break;
                                                    }
                                                    ::ichika::Status::Exit => {
                                                        break;
                                                    }
                                                    ::ichika::Status::Retry => {
                                                        break;
                                                    }
                                                    ::ichika::Status::RetryWith(
                                                        policy,
                                                        current_attempt,
                                                        retry_req,
                                                    ) => {
                                                        if current_attempt < policy.max_attempts {
                                                            std::thread::sleep(
                                                                std::time::Duration::from_millis(policy.delay_ms),
                                                            );
                                                            req = retry_req;
                                                            attempt = current_attempt + 1;
                                                            continue;
                                                        } else {
                                                            tx_response.send(retry_req).unwrap();
                                                            break;
                                                        }
                                                    }
                                                    ::ichika::Status::Panic(err) => {
                                                        {
                                                            ::std::io::_eprint(
                                                                format_args!(
                                                                    "Step {0} panicked: {1:?}\n",
                                                                    _step_0::id(),
                                                                    err,
                                                                ),
                                                            );
                                                        };
                                                        break;
                                                    }
                                                    ::ichika::Status::Switch((target, payload)) => {
                                                        if let Some(tx) = routing_table.get(target) {
                                                            tx.send(payload).unwrap();
                                                        } else {
                                                            {
                                                                ::std::io::_eprint(
                                                                    format_args!(
                                                                        "Warning: Switch target \'{0}\' not found or type incompatible, falling back to next step\n",
                                                                        target,
                                                                    ),
                                                                );
                                                            };
                                                            tx_response.send(payload).unwrap();
                                                        }
                                                        break;
                                                    }
                                                    ::ichika::Status::PanicSwitch((target, err)) => {
                                                        {
                                                            ::std::io::_eprint(
                                                                format_args!(
                                                                    "PanicSwitch to target \'{0}\' with error: {1:?}\n",
                                                                    target,
                                                                    err,
                                                                ),
                                                            );
                                                        };
                                                        break;
                                                    }
                                                    ::ichika::Status::Back((target, payload)) => {
                                                        if let Some(tx) = routing_table.get(target) {
                                                            tx.send(payload).unwrap();
                                                        } else {
                                                            {
                                                                ::std::io::_eprint(
                                                                    format_args!(
                                                                        "Warning: Back target \'{0}\' not found or type incompatible, falling back to next step\n",
                                                                        target,
                                                                    ),
                                                                );
                                                            };
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
                                pods__step_0.push(ThreadPod::new(_step_0::id(), thread));
                            }
                            let prev_pods_size = prev_pods_size + pods__step_0.len();
                            if !rx__step_1.is_empty()
                                && prev_pods_size + pods__step_1.len() < max_thread_count
                            {
                                let thread = std::thread::spawn({
                                    let rx_request = rx__step_1.clone();
                                    let tx_response = tx_pods_response.clone();
                                    let mut routing_table: ::std::collections::HashMap<
                                        &'static str,
                                        ::ichika::flume::Sender<String>,
                                    > = ::std::collections::HashMap::new();
                                    routing_table.insert("_step_0", tx__step_0.clone());
                                    move || {
                                        while let Ok(mut req) = rx_request.try_recv() {
                                            let mut attempt: usize = 0;
                                            loop {
                                                let res = _step_1::run(req);
                                                match res {
                                                    ::ichika::Status::Next(res) => {
                                                        tx_response.send(res).unwrap();
                                                        break;
                                                    }
                                                    ::ichika::Status::Exit => {
                                                        break;
                                                    }
                                                    ::ichika::Status::Retry => {
                                                        break;
                                                    }
                                                    ::ichika::Status::RetryWith(
                                                        policy,
                                                        current_attempt,
                                                        retry_req,
                                                    ) => {
                                                        if current_attempt < policy.max_attempts {
                                                            std::thread::sleep(
                                                                std::time::Duration::from_millis(policy.delay_ms),
                                                            );
                                                            req = retry_req;
                                                            attempt = current_attempt + 1;
                                                            continue;
                                                        } else {
                                                            tx_response.send(retry_req).unwrap();
                                                            break;
                                                        }
                                                    }
                                                    ::ichika::Status::Panic(err) => {
                                                        {
                                                            ::std::io::_eprint(
                                                                format_args!(
                                                                    "Step {0} panicked: {1:?}\n",
                                                                    _step_1::id(),
                                                                    err,
                                                                ),
                                                            );
                                                        };
                                                        break;
                                                    }
                                                    ::ichika::Status::Switch((target, payload)) => {
                                                        if let Some(tx) = routing_table.get(target) {
                                                            tx.send(payload).unwrap();
                                                        } else {
                                                            {
                                                                ::std::io::_eprint(
                                                                    format_args!(
                                                                        "Warning: Switch target \'{0}\' not found or type incompatible, falling back to next step\n",
                                                                        target,
                                                                    ),
                                                                );
                                                            };
                                                            tx_response.send(payload).unwrap();
                                                        }
                                                        break;
                                                    }
                                                    ::ichika::Status::PanicSwitch((target, err)) => {
                                                        {
                                                            ::std::io::_eprint(
                                                                format_args!(
                                                                    "PanicSwitch to target \'{0}\' with error: {1:?}\n",
                                                                    target,
                                                                    err,
                                                                ),
                                                            );
                                                        };
                                                        break;
                                                    }
                                                    ::ichika::Status::Back((target, payload)) => {
                                                        if let Some(tx) = routing_table.get(target) {
                                                            tx.send(payload).unwrap();
                                                        } else {
                                                            {
                                                                ::std::io::_eprint(
                                                                    format_args!(
                                                                        "Warning: Back target \'{0}\' not found or type incompatible, falling back to next step\n",
                                                                        target,
                                                                    ),
                                                                );
                                                            };
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
                                pods__step_1.push(ThreadPod::new(_step_1::id(), thread));
                            }
                            let prev_pods_size = prev_pods_size + pods__step_1.len();
                            if rx_thread_usage_request.try_recv().is_ok() {
                                tx_thread_usage_response
                                    .send(pods__step_0.len() + pods__step_1.len())
                                    .unwrap();
                            }
                            if rx_task_count_request.try_recv().is_ok() {
                                tx_task_count_response
                                    .send(
                                        pods__step_0.len() + pods__step_1.len() + tx__step_0.len(),
                                    )
                                    .unwrap();
                            }
                            if rx_shutdown.try_recv().is_ok() {
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                        loop {
                            pods__step_0.retain(|pod: &ThreadPod| pod.is_alive());
                            pods__step_1.retain(|pod: &ThreadPod| pod.is_alive());
                            if pods__step_0.is_empty() && pods__step_1.is_empty() {
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
                    tx_send_request: tx__step_0.clone(),
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
    };
}
