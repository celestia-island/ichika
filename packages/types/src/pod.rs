use anyhow::Result;
use std::thread::JoinHandle;

pub struct ThreadPod {
    step: String,
    thread: JoinHandle<Result<()>>,
}

impl ThreadPod {
    pub fn new(step: impl ToString, thread: JoinHandle<Result<()>>) -> Self {
        Self {
            step: step.to_string(),
            thread,
        }
    }

    pub fn step(&self) -> &str {
        &self.step
    }

    pub fn is_alive(&self) -> bool {
        !self.thread.is_finished()
    }
}
