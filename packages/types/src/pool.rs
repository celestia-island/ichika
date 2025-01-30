pub trait ThreadPool {
    type Request: Clone;
    type Response: Clone;

    fn send(&mut self, req: Self::Request);
    fn recv(&mut self) -> Option<Self::Response>;
    fn thread_usage(&self) -> usize;
    fn task_count(&self) -> usize;
}
