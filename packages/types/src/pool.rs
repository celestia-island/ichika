pub trait ThreadPool {
    type Request: Clone;
    type Response: Clone;

    fn send(&mut self, req: Self::Request);
    fn recv(&mut self) -> Option<Self::Response>;
    fn thread_usage(&self) -> usize; // 看每一级有多少个线程正在工作
    fn task_count(&self) -> usize; // 看每一级积压了多少任务，其中 Finally 是最后有多少结果没取走
}
