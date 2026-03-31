use ::ichika::status::IntoStatus;

struct _step_0;
impl ::ichika::node::ThreadNode for _step_0 {
    type Request = String;
    type Response = usize;
    fn run(req: Self::Request) -> ::ichika::Status<Self::Response, ::ichika::anyhow::Error> {
        { Ok(req.len()) }.into_status()
    }
}
impl ::ichika::node::ThreadNodeEnum for _step_0 {
    fn id() -> &'static str {
        stringify!(_step_0)
    }
}

struct _step_1;
impl ::ichika::node::ThreadNode for _step_1 {
    type Request = usize;
    type Response = String;
    fn run(req: Self::Request) -> ::ichika::Status<Self::Response, ::ichika::anyhow::Error> {
        { Ok(req.to_string()) }.into_status()
    }
}
impl ::ichika::node::ThreadNodeEnum for _step_1 {
    fn id() -> &'static str {
        stringify!(_step_1)
    }
}

struct _Pool {
    tx_send_request: ::ichika::flume::Sender<String>,
    rx_recv_response: ::ichika::flume::Receiver<String>,
}

impl ::ichika::pool::ThreadPool for _Pool {
    type Request = String;
    type Response = String;

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
        Ok(0)
    }

    fn task_count(&self, _stage: impl ToString) -> ::ichika::anyhow::Result<usize> {
        Ok(0)
    }
}

impl _Pool {
    pub fn new() -> ::ichika::anyhow::Result<Self> {
        let (tx__step_0, _rx__step_0) = ::ichika::flume::unbounded::<String>();
        let (_tx__step_1, _rx__step_1) = ::ichika::flume::unbounded::<usize>();
        let (_tx_pods_response, rx_pods_response) = ::ichika::flume::unbounded::<String>();

        Ok(Self {
            tx_send_request: tx__step_0,
            rx_recv_response: rx_pods_response,
        })
    }
}

fn main() -> ::ichika::anyhow::Result<()> {
    let _pool = _Pool::new()?;
    Ok(())
}
