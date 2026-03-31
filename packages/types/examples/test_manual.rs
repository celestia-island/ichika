use anyhow::Result;
use ichika::pool::ThreadPool;
use ichika::status::IntoStatus;

// Manually simulate what the macro generates
fn main() -> Result<()> {
    struct _step_0;
    impl ichika::node::ThreadNode for _step_0 {
        type Request = String;
        type Response = usize;
        fn run(req: Self::Request) -> ichika::Status<Self::Response, anyhow::Error> {
            Ok(req.len()).into_status()
        }
    }
    impl ichika::node::ThreadNodeEnum for _step_0 {
        fn id() -> &'static str { "_step_0" }
    }
    
    struct _step_1;
    impl ichika::node::ThreadNode for _step_1 {
        type Request = usize;
        type Response = String;
        fn run(req: Self::Request) -> ichika::Status<Self::Response, anyhow::Error> {
            Ok(req.to_string()).into_status()
        }
    }
    impl ichika::node::ThreadNodeEnum for _step_1 {
        fn id() -> &'static str { "_step_1" }
    }
    
    struct _Pool {
        tx_send_request: ichika::flume::Sender<String>,
        rx_recv_response: ichika::flume::Receiver<String>,
    }
    
    impl ThreadPool for _Pool {
        type Request = String;
        type Response = String;
        
        fn send(&self, req: Self::Request) -> anyhow::Result<()> {
            self.tx_send_request.send(req)?;
            Ok(())
        }
        
        fn recv(&self) -> anyhow::Result<Option<Self::Response>> {
            Ok(self.rx_recv_response.try_recv().ok())
        }
        
        fn thread_usage(&self) -> anyhow::Result<usize> {
            Ok(0)
        }
        
        fn task_count(&self, id: impl ToString) -> anyhow::Result<usize> {
            Ok(0)
        }
    }
    
    impl _Pool {
        fn new() -> anyhow::Result<Self> {
            let (tx_send_request, rx_recv_response) = ichika::flume::unbounded();
            Ok(Self { tx_send_request, rx_recv_response })
        }
    }
    
    let _pool = _Pool::new()?;
    Ok(())
}
