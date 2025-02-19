pub trait ThreadPool {
    type Request: Clone;
    type Response: Clone;

    fn send(&self, req: Self::Request) -> crate::Result<()>;
    fn recv(&self) -> crate::Result<Option<Self::Response>>;

    fn thread_usage(&self) -> crate::Result<usize>;
    fn task_count(&self, id: impl ToString) -> crate::Result<usize>;
}
