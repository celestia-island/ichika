use crate::pod::ThreadPod;

pub trait ThreadNode {
    type Request: Clone;
    type Response: Clone;

    fn run(&self) -> ThreadPod<Self::Request, Self::Response>;
}

#[macro_export]
macro_rules! create_node {
    ($struct_name: ident |$arg_name: ident: $arg_ty: ty| -> $ret_ty: ty { $($closure: tt)* }) => {
        struct $struct_name {
            tx_request: ::flume::Sender<$arg_ty>,
            tx_response: ::flume::Sender<$ret_ty>,
            rx_request: ::flume::Receiver<$arg_ty>,
            rx_response: ::flume::Receiver<$ret_ty>,

            thread_count_limit: Option<usize>,
            pods: Vec<::ichika::ThreadPod<$arg_ty, $ret_ty>>,
        }

        impl Default for $struct_name {
            fn default() -> Self {
                let (tx_request, rx_request) = ::flume::unbounded();
                let (tx_response, rx_response) = ::flume::unbounded();

                Self {
                    tx_request,
                    tx_response,
                    rx_request,
                    rx_response,

                    thread_count_limit: None,
                    pods: Vec::new(),
                }
            }
        }

        impl $struct_name {
            pub fn with_limit(self, limit: usize) -> Self {
                Self {
                    thread_count_limit: Some(limit),
                    ..self
                }
            }
        }

        impl ::ichika::ThreadNode for $struct_name {
            type Request = $arg_ty;
            type Response = $ret_ty;

            fn run(&self) -> ::ichika::ThreadPod<Self::Request, Self::Response> {
                let tx_request = self.tx_request.clone();
                let tx_response = self.tx_response.clone();
                let rx_request = self.rx_request.clone();
                let rx_response = self.rx_response.clone();

                let thread = std::thread::spawn(move || {
                    while let Ok($arg_name) = rx_request.recv() {
                        let response = $($closure)*;
                        tx_response.send(response).unwrap();
                    }

                    Ok(())
                });
                ThreadPod::new(
                    tx_request,
                    rx_response,
                    thread,
                )
            }
        }

        impl $struct_name {
            fn clean_up(&mut self) {
                self.pods.retain(|pod| pod.is_alive());
            }

            pub fn send(&mut self, request: <$struct_name as ::ichika::ThreadNode>::Request) -> ::anyhow::Result<()> {
                self.clean_up();

                let thread_count_limit = self.thread_count_limit.unwrap_or(::num_cpus::get());
                if self.pods.len() < thread_count_limit {
                    let pod = self.run();
                    self.pods.push(pod);
                }

                self.tx_request
                    .send(request)
                    .map_err(|err| ::anyhow::anyhow!("Failed to send request to thread: {}", err))?;
                Ok(())
            }

            pub fn recv(&self) -> ::anyhow::Result<<$struct_name as ::ichika::ThreadNode>::Response> {
                self.rx_response.recv().map_err(|err| err.into())
            }
        }
    };
}
