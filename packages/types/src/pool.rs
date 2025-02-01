pub trait ThreadPool {
    type Request: Clone;
    type Response: Clone;

    fn send(&mut self, req: Self::Request);
    fn recv(&mut self) -> Option<Self::Response>;

    fn set_task_limit(&mut self, limit: usize, id: impl ToString);
    fn set_task_limit_all_threads(&mut self, limit: usize);
    fn set_thread_count_limit(&mut self, limit: usize);

    fn thread_usage(&self) -> usize;
    fn task_count(&self, id: impl ToString) -> usize;
}
