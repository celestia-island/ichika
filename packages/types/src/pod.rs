use anyhow::Result;
use std::thread::JoinHandle;

pub struct ThreadPod {
    stage: String,
    thread: JoinHandle<Result<()>>,
}

impl ThreadPod {
    pub fn new(stage: impl ToString, thread: JoinHandle<Result<()>>) -> Self {
        Self {
            stage: stage.to_string(),
            thread,
        }
    }

    pub fn stage(&self) -> &str {
        &self.stage
    }

    pub fn is_alive(&self) -> bool {
        !self.thread.is_finished()
    }
}
