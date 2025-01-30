use anyhow::{anyhow, Result};
use std::thread::JoinHandle;

pub struct ThreadPod<I, O> {
    tx_request: flume::Sender<I>,
    rx_response: flume::Receiver<O>,
    thread: JoinHandle<Result<()>>,
}

impl<I, O> ThreadPod<I, O> {
    pub fn new(
        tx_request: flume::Sender<I>,
        rx_response: flume::Receiver<O>,
        thread: JoinHandle<Result<()>>,
    ) -> Self {
        Self {
            tx_request,
            rx_response,
            thread,
        }
    }

    pub fn send(&self, request: I) -> Result<()> {
        self.tx_request
            .send(request)
            .map_err(|err| anyhow!("Failed to send request to thread: {}", err))?;
        Ok(())
    }

    pub fn recv(&self) -> Result<O> {
        self.rx_response.recv().map_err(|err| err.into())
    }

    pub fn is_alive(&self) -> bool {
        !self.thread.is_finished()
    }
}
