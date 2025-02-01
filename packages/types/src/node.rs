pub trait ThreadNode {
    type Request: Clone;
    type Response: Clone;

    fn run(req: Self::Request) -> Self::Response;
}

#[async_trait::async_trait]
pub trait ThreadNodeAsync {
    type Request: Clone;
    type Response: Clone;

    async fn run(req: Self::Request) -> Self::Response;
}

pub trait ThreadNodeEnum {
    fn id(&self) -> &str;
}
